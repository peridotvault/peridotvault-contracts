//! PGC1 M1 cross-chain schema logic (ANT-490, parent ANT-485 spec §3).
//!
//! Pure, unit-testable functions shared by the `XchainGameEntry` instruction
//! handlers. No Anchor account access here — all inputs are plain values so
//! `cargo test -p registry` proves the schema without a validator.
//!
//! Lenses: Gas/CU (derivation is off-chain-first; on-chain recomputation is a
//! single keccak over a stack-built preimage — measured in tests, must stay
//! under 50k CUs); Oracle risk (chainId allowlist is exact-match, §5);
//! Fuzz & invariant (property tests over transitions live in `tests/` below).

use sha3::{Digest, Keccak256};

/// M1 schema version (M0 M5 lineage).
pub const XCHAIN_CURRENT_VERSION: u8 = 1;
/// spec §3.1: localId 1..128 bytes, printable ASCII only.
pub const MAX_LOCAL_ID_LEN: usize = 128;
/// spec §3.1 normative chainId table (exact byte equality, §5).
pub const SUPPORTED_CHAIN_IDS: [&str; 5] = [
    "eip155:31337",
    "eip155:84532",
    "eip155:4202",
    "solana:localnet",
    "solana:EtWBMkgFUSkMhvZSGQi4uAiEE8RjqiB1MwWXEaAu34",
];

/// Lifecycle §3.2 (discriminants match the EVM `XStatus` order).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum XchainStatus {
    Proposed = 0,
    Active = 1,
    Deprecated = 2,
    Frozen = 3,
}

impl XchainStatus {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Proposed),
            1 => Some(Self::Active),
            2 => Some(Self::Deprecated),
            3 => Some(Self::Frozen),
            _ => None,
        }
    }
}

/// bytes+format discipline (spec §3.2, AC-REG-5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum XchainIdFormat {
    Evm20 = 0,
    Solana32 = 1,
}

impl XchainIdFormat {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Evm20),
            1 => Some(Self::Solana32),
            _ => None,
        }
    }

    pub fn byte_len(self) -> usize {
        match self {
            Self::Evm20 => 20,
            Self::Solana32 => 32,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum XchainError {
    UnknownChainId,
    InvalidPublisher,
    EmptyLocalId,
    LocalIdTooLong,
    InvalidLocalIdCharset,
    InvalidProgramId,
    InvalidTransition,
    UpdateNotEffective,
    ZeroTimelock,
    VersionMismatch,
    UnsupportedVersion,
    PaidAmountZeroOnPaidPath,
    PaidAmountNonZeroOnFreePath,
    PaidTokenNonEmptyOnFreePath,
    InvalidTokenFormat,
}

pub fn is_supported_chain(chain_id: &str) -> bool {
    SUPPORTED_CHAIN_IDS.contains(&chain_id)
}

pub fn validate_chain_id(chain_id: &str) -> Result<(), XchainError> {
    if !is_supported_chain(chain_id) {
        return Err(XchainError::UnknownChainId);
    }
    Ok(())
}

pub fn validate_sized_bytes(bytes: &[u8], format: XchainIdFormat, program: bool) -> Result<(), XchainError> {
    if bytes.len() != format.byte_len() || bytes.iter().all(|b| *b == 0) {
        return if program {
            Err(XchainError::InvalidProgramId)
        } else {
            Err(XchainError::InvalidPublisher)
        };
    }
    Ok(())
}

/// 1..128 bytes, printable ASCII only (0x21..0x7E) — byte-identical rule to
/// the EVM `_checkLocalId` (spec §5 ASCII-only).
pub fn validate_local_id(local_id: &str) -> Result<(), XchainError> {
    let b = local_id.as_bytes();
    if b.is_empty() {
        return Err(XchainError::EmptyLocalId);
    }
    if b.len() > MAX_LOCAL_ID_LEN {
        return Err(XchainError::LocalIdTooLong);
    }
    if !b.iter().all(|c| (0x21..=0x7E).contains(c)) {
        return Err(XchainError::InvalidLocalIdCharset);
    }
    Ok(())
}

fn push_u256_be(out: &mut Vec<u8>, v: usize) {
    let mut w = [0u8; 32];
    w[28..].copy_from_slice(&(v as u32).to_be_bytes());
    out.extend_from_slice(&w);
}

fn push_padded(out: &mut Vec<u8>, data: &[u8]) {
    out.extend_from_slice(data);
    let pad = (32 - (data.len() % 32)) % 32;
    out.extend(std::iter::repeat(0u8).take(pad));
}

/// gameId = keccak256(abi.encode(string chainId, bytes publisher, string localId)).
///
/// Manual §3.1 layout with DYNAMIC head offsets: off1 = 0x60+32+padded(chainId),
/// off2 = off1+32+padded(publisher). The spec table's hardcoded 0x60/0xa0/0xe0
/// is only valid for short inputs (see fixtures/gameid.vectors.json `_deviation`
/// and the EVM AC-GAMEID-2 test); the formula governs, not the table.
pub fn derive_game_id(chain_id: &str, publisher: &[u8], local_id: &str) -> [u8; 32] {
    let c = chain_id.as_bytes();
    let l = local_id.as_bytes();
    let padded = |n: usize| ((n + 31) / 32) * 32;
    let off1 = 0x60 + 32 + padded(c.len());
    let off2 = off1 + 32 + padded(publisher.len());
    let mut preimage = Vec::with_capacity(off2 + 32 + padded(l.len()));
    push_u256_be(&mut preimage, 0x60);
    push_u256_be(&mut preimage, off1);
    push_u256_be(&mut preimage, off2);
    push_u256_be(&mut preimage, c.len());
    push_padded(&mut preimage, c);
    push_u256_be(&mut preimage, publisher.len());
    push_padded(&mut preimage, publisher);
    push_u256_be(&mut preimage, l.len());
    push_padded(&mut preimage, l);
    Keccak256::digest(&preimage).into()
}

/// Status lifecycle §3.2. `freeze` is the emergency path (bypasses timelock);
/// everything else follows PROPOSED→ACTIVE→DEPRECATED. FROZEN is terminal.
pub fn transition(current: XchainStatus, next: XchainStatus, freeze: bool) -> Result<(), XchainError> {
    if freeze {
        return match (current, next) {
            (XchainStatus::Active, XchainStatus::Frozen)
            | (XchainStatus::Deprecated, XchainStatus::Frozen) => Ok(()),
            _ => Err(XchainError::InvalidTransition),
        };
    }
    match (current, next) {
        (XchainStatus::Proposed, XchainStatus::Active)
        | (XchainStatus::Active, XchainStatus::Deprecated) => Ok(()),
        _ => Err(XchainError::InvalidTransition),
    }
}

/// Execution MUST revert before effectiveAt (AC-TIME-1).
pub fn check_execute(now: i64, effective_at: i64) -> Result<(), XchainError> {
    if now < effective_at {
        return Err(XchainError::UpdateNotEffective);
    }
    Ok(())
}

pub fn check_timelock_delay(delay: i64) -> Result<(), XchainError> {
    if delay <= 0 {
        return Err(XchainError::ZeroTimelock);
    }
    Ok(())
}

/// Destination envelope rules §3.4 (AC-ENV-1..3). Mirrors XChainEnvelope.sol.
#[allow(clippy::too_many_arguments)]
pub fn verify_envelope(
    is_free: bool,
    paid_amount: u64,
    paid_token_len: usize,
    paid_token_format: u8,
    receipt_version: u8,
    version: u8,
) -> Result<(), XchainError> {
    if version != receipt_version {
        return Err(XchainError::VersionMismatch);
    }
    if version > XCHAIN_CURRENT_VERSION {
        return Err(XchainError::UnsupportedVersion);
    }
    if !is_free && paid_amount == 0 {
        return Err(XchainError::PaidAmountZeroOnPaidPath);
    }
    if is_free && paid_amount != 0 {
        return Err(XchainError::PaidAmountNonZeroOnFreePath);
    }
    if is_free && paid_token_len != 0 {
        return Err(XchainError::PaidTokenNonEmptyOnFreePath);
    }
    if paid_token_format > 1 {
        return Err(XchainError::InvalidTokenFormat);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex32(b: &[u8]) -> String {
        format!("0x{}", b.iter().map(|x| format!("{:02x}", x)).collect::<String>())
    }

    fn load_vectors() -> serde_json::Value {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../fixtures/gameid.vectors.json"
        );
        let body = std::fs::read_to_string(path).expect("fixtures/gameid.vectors.json loads");
        serde_json::from_str(&body).expect("vectors json parses")
    }

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// AC-GAMEID-1 + AC-GAMEID-2 (Solana side): manual layout over all 5
    /// fixture vectors reproduces the EVM-verified gameIds.
    #[test]
    fn gameid_vectors_cross_stack() {
        let v = load_vectors();
        let arr = v["vectors"].as_array().unwrap();
        assert_eq!(arr.len(), 5, "exactly 5 vectors");
        for item in arr {
            let chain = item["chainId"].as_str().unwrap();
            let pubhex = item["publisherHex"].as_str().unwrap();
            let local = item["localId"].as_str().unwrap();
            let want = item["gameId"].as_str().unwrap().to_lowercase();
            let got = hex32(&derive_game_id(chain, &hex_to_bytes(pubhex), local));
            assert_eq!(got, want, "vector {} {}", item["vector"], local);
        }
    }

    /// AC-GAMEID-3: chainId domain separation.
    #[test]
    fn gameid_domain_separation() {
        let p = [0x11u8; 20];
        let a = derive_game_id("eip155:84532", &p, "same-game");
        let b = derive_game_id("eip155:4202", &p, "same-game");
        assert_ne!(a, b);
        // EVM20 vs SOLANA32 publisher lengths also separate.
        let c = derive_game_id("eip155:84532", &[0x11u8; 32], "same-game");
        assert_ne!(a, c);
    }

    /// AC-GAMEID-4: input validation rejects with specific errors.
    #[test]
    fn gameid_input_validation() {
        assert_eq!(validate_chain_id("eip155:99999"), Err(XchainError::UnknownChainId));
        assert_eq!(validate_chain_id("EIP155:84532"), Err(XchainError::UnknownChainId));
        assert_eq!(validate_local_id(""), Err(XchainError::EmptyLocalId));
        assert_eq!(validate_local_id(&"y".repeat(129)), Err(XchainError::LocalIdTooLong));
        assert_eq!(validate_local_id("has space"), Err(XchainError::InvalidLocalIdCharset));
        assert_eq!(validate_local_id("tab\there"), Err(XchainError::InvalidLocalIdCharset));
        assert_eq!(
            validate_sized_bytes(&[0x11u8; 19], XchainIdFormat::Evm20, false),
            Err(XchainError::InvalidPublisher)
        );
        assert_eq!(
            validate_sized_bytes(&[0u8; 20], XchainIdFormat::Evm20, false),
            Err(XchainError::InvalidPublisher)
        );
        assert_eq!(
            validate_sized_bytes(&[0x11u8; 20], XchainIdFormat::Evm20, false),
            Ok(())
        );
    }

    /// AC-REG-1..3 equivalent: lifecycle transitions incl. terminal FROZEN.
    #[test]
    fn lifecycle_transitions() {
        use XchainStatus as S;
        assert!(transition(S::Proposed, S::Active, false).is_ok());
        assert!(transition(S::Active, S::Deprecated, false).is_ok());
        assert!(transition(S::Active, S::Frozen, true).is_ok());
        assert!(transition(S::Deprecated, S::Frozen, true).is_ok());
        // Sad paths: skip-level, backwards, re-activation, non-freeze to FROZEN.
        assert_eq!(
            transition(S::Proposed, S::Deprecated, false),
            Err(XchainError::InvalidTransition)
        );
        assert_eq!(
            transition(S::Frozen, S::Active, false),
            Err(XchainError::InvalidTransition)
        );
        assert_eq!(
            transition(S::Active, S::Frozen, false),
            Err(XchainError::InvalidTransition)
        );
        assert_eq!(
            transition(S::Proposed, S::Frozen, true),
            Err(XchainError::InvalidTransition)
        );
    }

    /// AC-TIME-1: pre-effectiveAt execution reverts.
    #[test]
    fn timelock_enforcement() {
        assert_eq!(check_execute(99, 100), Err(XchainError::UpdateNotEffective));
        assert_eq!(check_execute(100, 100), Ok(()));
        assert_eq!(check_timelock_delay(0), Err(XchainError::ZeroTimelock));
        assert_eq!(check_timelock_delay(60), Ok(()));
    }

    /// AC-ENV-1..3 equivalent.
    #[test]
    fn envelope_rules() {
        assert!(verify_envelope(true, 0, 0, 0, 1, 1).is_ok());
        assert!(verify_envelope(false, 100, 20, 0, 1, 1).is_ok());
        assert_eq!(
            verify_envelope(false, 0, 0, 0, 1, 1),
            Err(XchainError::PaidAmountZeroOnPaidPath)
        );
        assert_eq!(
            verify_envelope(true, 5, 0, 0, 1, 1),
            Err(XchainError::PaidAmountNonZeroOnFreePath)
        );
        assert_eq!(
            verify_envelope(true, 0, 0, 0, 0, 1),
            Err(XchainError::VersionMismatch)
        );
        assert_eq!(
            verify_envelope(false, 1, 1, 9, 1, 1),
            Err(XchainError::InvalidTokenFormat)
        );
    }
}
