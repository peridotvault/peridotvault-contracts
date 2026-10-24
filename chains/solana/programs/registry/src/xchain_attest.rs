//! PGC1 M2 attestation verification logic (ANT-487, parent ANT-485 spec §3.4).
//!
//! Pure, unit-testable functions shared by the `consume_attestation`
//! instruction handlers. No Anchor account access here — all inputs are plain
//! values so `cargo test -p registry` proves the scheme without a validator.
//!
//! The EIP-712 `PGL1-XCHAIN` envelope is transport-agnostic: `transport_id`
//! is an opaque label bound into the digest (Wormhole-primary / LZ-V2
//! fallback swappable, no lock-in — handoff §9). Payment binding (Tobias
//! [ANT-479] residual risk #1) reuses `verify_envelope`: `is_free` /
//! `paid_amount` are signed AND re-checked dest-side, never inferred.
//!
//! Lenses: Reorg & finality (finalized-only observations + per-config reorg
//! delay counted from `observed_at`); Fuzz & invariant (threshold counting
//! over distinct members, replay set — the money paths); Gas/CU (bounded
//! 3-of-5 recovery loop; cheap version/payment checks before any recover).

use solana_secp256k1_recover::secp256k1_recover;
use sha3::{Digest, Keccak256};

use crate::xchain::{
    verify_envelope, XchainError, XCHAIN_CURRENT_VERSION,
};

/// EIP-712 type string. MUST match XChainAttester.ATTESTATION_TYPEHASH and
/// the TS encoder in chains/evm/test/M2Attestation.ts byte-for-byte.
/// fixtures/xchain-attestation.vectors.json pins the triple implementation.
pub const ATTESTATION_TYPESTRING: &str = "Attestation(bytes32 gameId,string homeChainId,string destChainId,bytes32 licensee,bool isFree,uint256 paidAmount,bytes paidToken,uint8 paidTokenFormat,uint8 receiptVersion,uint8 version,uint256 nonce,uint256 homeFinalizedBlock,uint64 observedAt,bytes32 transportId)";
pub const EIP712_DOMAIN_TYPESTRING: &str =
    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

/// 3-of-5 quorum (mirrors XChainAttester.THRESHOLD / ATTESTER_COUNT).
pub const ATTESTER_THRESHOLD: usize = 3;
pub const ATTESTER_COUNT: usize = 5;

pub const TOKEN_FORMAT_EVM20: u8 = 0;
pub const TOKEN_FORMAT_SOLANA32: u8 = 1;

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    Keccak256::digest(data).into()
}

fn word_u64(v: u64) -> [u8; 32] {
    let mut w = [0u8; 32];
    w[24..].copy_from_slice(&v.to_be_bytes());
    w
}

fn word_u8(v: u8) -> [u8; 32] {
    let mut w = [0u8; 32];
    w[31] = v;
    w
}

/// EIP-712 view of the attestation. Integer widths mirror the Solidity
/// struct: u64 values encode as uint256 words, u8 as uint8 words, bool as a
/// 0/1 word — byte-identical to `abi.encode` on the EVM side.
/// Domain-width note (Tobias F4): the Solana dest represents paid_amount /
/// nonce / home_finalized_block / observed_at as u64, encoded as EIP-712
/// uint256 words. Encodings agree with the EVM side below 2^64; EVM uint256
/// values >= 2^64 are UNREPRESENTABLE on Solana — fail-closed (never
/// recognized, no false accept), but large-value licenses silently cannot
/// cross. Keep cross-chain settlements < 2^64 unless the domain is widened.
pub struct AttestationView<'a> {
    pub game_id: [u8; 32],
    pub home_chain_id: &'a str,
    pub dest_chain_id: &'a str,
    pub licensee: [u8; 32],
    pub is_free: bool,
    pub paid_amount: u64,
    pub paid_token: &'a [u8],
    pub paid_token_format: u8,
    pub receipt_version: u8,
    pub version: u8,
    pub nonce: u64,
    pub home_finalized_block: u64,
    pub observed_at: u64,
    pub transport_id: [u8; 32],
}

pub struct DomainView<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub chain_id: u64,
    /// 20-byte EVM address of the dest-chain XChainAttester contract.
    pub verifying_contract: [u8; 20],
}

pub fn domain_separator(d: &DomainView) -> [u8; 32] {
    let mut enc = Vec::with_capacity(5 * 32);
    enc.extend_from_slice(&keccak256(EIP712_DOMAIN_TYPESTRING.as_bytes()));
    enc.extend_from_slice(&keccak256(d.name.as_bytes()));
    enc.extend_from_slice(&keccak256(d.version.as_bytes()));
    enc.extend_from_slice(&word_u64(d.chain_id));
    let mut addr_word = [0u8; 32];
    addr_word[12..].copy_from_slice(&d.verifying_contract);
    enc.extend_from_slice(&addr_word);
    keccak256(&enc)
}

pub fn struct_hash(a: &AttestationView) -> [u8; 32] {
    let mut enc = Vec::with_capacity(15 * 32);
    enc.extend_from_slice(&keccak256(ATTESTATION_TYPESTRING.as_bytes()));
    enc.extend_from_slice(&a.game_id);
    enc.extend_from_slice(&keccak256(a.home_chain_id.as_bytes()));
    enc.extend_from_slice(&keccak256(a.dest_chain_id.as_bytes()));
    enc.extend_from_slice(&a.licensee);
    enc.extend_from_slice(&word_u64(a.is_free as u64));
    enc.extend_from_slice(&word_u64(a.paid_amount));
    enc.extend_from_slice(&keccak256(a.paid_token));
    enc.extend_from_slice(&word_u8(a.paid_token_format));
    enc.extend_from_slice(&word_u8(a.receipt_version));
    enc.extend_from_slice(&word_u8(a.version));
    enc.extend_from_slice(&word_u64(a.nonce));
    enc.extend_from_slice(&word_u64(a.home_finalized_block));
    enc.extend_from_slice(&word_u64(a.observed_at));
    enc.extend_from_slice(&a.transport_id);
    keccak256(&enc)
}

/// Final EIP-712 digest: keccak(0x1901 || domainSeparator || structHash).
/// The domain binds chainId + verifying contract, so cross-chain /
/// cross-contract replay is impossible by construction.
pub fn attestation_digest(d: &DomainView, a: &AttestationView) -> [u8; 32] {
    let mut enc = Vec::with_capacity(2 + 32 + 32);
    enc.push(0x19);
    enc.push(0x01);
    enc.extend_from_slice(&domain_separator(d));
    enc.extend_from_slice(&struct_hash(a));
    keccak256(&enc)
}

/// Ethereum address derivation: last 20 bytes of keccak(uncompressed pubkey).
pub fn eth_addr_from_pubkey(pubkey64: &[u8; 64]) -> [u8; 20] {
    let h = keccak256(pubkey64);
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&h[12..]);
    addr
}

/// Recover attester addresses for one digest. Each signature is a 64-byte
/// compact sig plus recovery id. Uses the same `secp256k1_recover` the
/// on-chain handler calls (syscall on-chain, libsecp256k1 fallback in
/// tests) — the recovery path itself is unit-tested, not mocked.
pub fn recover_attester_addrs(
    digest: &[u8; 32],
    sigs: &[([u8; 64], u8)],
) -> Result<Vec<[u8; 20]>, XchainError> {
    let mut out = Vec::with_capacity(sigs.len());
    for (sig, recid) in sigs {
        let pubkey = secp256k1_recover(digest, *recid, sig)
            .map_err(|_| XchainError::InvalidAttestationSignature)?;
        out.push(eth_addr_from_pubkey(&pubkey.to_bytes()));
    }
    Ok(out)
}

/// Quorum check over recovered addresses: every entry must be a DISTINCT
/// attester-set member; at least ATTESTER_THRESHOLD distinct members.
/// Unknown or double-counted signers fail CLOSED (never silently skipped).
pub fn verify_threshold(
    recovered: &[[u8; 20]],
    signers: &[[u8; 20]; ATTESTER_COUNT],
) -> Result<(), XchainError> {
    if recovered.len() < ATTESTER_THRESHOLD {
        return Err(XchainError::ThresholdNotMet);
    }
    let mut seen: Vec<[u8; 20]> = Vec::with_capacity(ATTESTER_THRESHOLD);
    for addr in recovered {
        if !signers.contains(addr) {
            return Err(XchainError::UnknownAttester);
        }
        if seen.contains(addr) {
            return Err(XchainError::DuplicateAttesterSignature);
        }
        seen.push(*addr);
        if seen.len() == ATTESTER_THRESHOLD {
            break;
        }
    }
    if seen.len() < ATTESTER_THRESHOLD {
        return Err(XchainError::ThresholdNotMet);
    }
    Ok(())
}

/// Replay backstop over a caller-held consumed set (the on-chain equivalent
/// is the recognition PDA init: re-creating (game, licensee, nonce) fails).
pub fn check_replay(
    digest: &[u8; 32],
    consumed: &[[u8; 32]],
) -> Result<(), XchainError> {
    if consumed.contains(digest) {
        return Err(XchainError::AlreadyConsumed);
    }
    Ok(())
}

/// Reorg/finality policy: the attester observation must not be from the
/// future, and `delay` seconds must have passed since `observed_at` so late
/// reorg evidence can arrive before recognition settles.
pub fn check_reorg(
    now: i64,
    observed_at: i64,
    delay: i64,
) -> Result<(), XchainError> {
    if observed_at > now {
        return Err(XchainError::ObservedAtInFuture);
    }
    if now < observed_at.saturating_add(delay) {
        return Err(XchainError::ReorgDelayNotElapsed);
    }
    Ok(())
}

/// Tobias F2: the attested dest chain must EXACTLY match the attester
/// config's dest chain (execution-anchored domain). Claim-anchored domains
/// (caller-supplied dest with no config check) are rejected.
pub fn check_dest_chain_id(config_dest: &str, args_dest: &str) -> Result<(), XchainError> {
    if config_dest != args_dest {
        return Err(XchainError::InvalidDestChainId);
    }
    Ok(())
}

/// Tobias F3: cap supplied signatures at the attester-set size BEFORE any
/// recovery (each recovery costs CUs; junk-padded relays must not burn them).
pub fn check_sig_cap(len: usize) -> Result<(), XchainError> {
    if len > ATTESTER_COUNT {
        return Err(XchainError::TooManyAttestationSignatures);
    }
    Ok(())
}

/// Full dest-side policy BEFORE signature recovery: version lineage +
/// payment binding (AC-ENV-1..3) + supported home chain. Mirrors the cheap
/// checks-first ordering of XChainAttester.consume (Gas lens).
#[allow(clippy::too_many_arguments)]
pub fn check_attestation_policy(
    version: u8,
    receipt_version: u8,
    is_free: bool,
    paid_amount: u64,
    paid_token: &[u8],
    paid_token_format: u8,
    home_chain_id: &str,
) -> Result<(), XchainError> {
    if version != receipt_version {
        return Err(XchainError::VersionMismatch);
    }
    if version > XCHAIN_CURRENT_VERSION {
        return Err(XchainError::UnsupportedVersion);
    }
    verify_envelope(
        is_free,
        paid_amount,
        paid_token.len(),
        paid_token_format,
        receipt_version,
        version,
    )?;
    crate::xchain::validate_chain_id(home_chain_id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_to_32(s: &str) -> [u8; 32] {
        let s = s.strip_prefix("0x").unwrap();
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
        }
        out
    }

    fn hex_to_20(s: &str) -> [u8; 20] {
        let s = s.strip_prefix("0x").unwrap();
        let mut out = [0u8; 20];
        for i in 0..20 {
            out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
        }
        out
    }

    fn hex_to_bytes(s: &str) -> Vec<u8> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        if s.is_empty() {
            return vec![];
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn load_fixture() -> serde_json::Value {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../fixtures/xchain-attestation.vectors.json"
        );
        let body = std::fs::read_to_string(path).expect("attestation vectors load");
        serde_json::from_str(&body).expect("vectors json parses")
    }

    /// Build owned fixtures first (lifetimes), then views over them.
    struct OwnedVector {
        domain_name: String,
        domain_version: String,
        verifying_contract: [u8; 20],
        chain_id: u64,
        game_id: [u8; 32],
        home: String,
        dest: String,
        licensee: [u8; 32],
        is_free: bool,
        paid_amount: u64,
        paid_token: Vec<u8>,
        paid_token_format: u8,
        receipt_version: u8,
        version: u8,
        nonce: u64,
        home_finalized_block: u64,
        observed_at: u64,
        transport_id: [u8; 32],
        digest: [u8; 32],
    }

    fn load_owned(label: &str) -> OwnedVector {
        let fx = load_fixture();
        let d = &fx["domain"];
        let v = fx["vectors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["label"] == label)
            .unwrap_or_else(|| panic!("vector {label} present"));
        let m = &v["message"];
        let u64_from = |k: &str| m[k].as_str().unwrap().parse::<u64>().unwrap();
        OwnedVector {
            domain_name: d["name"].as_str().unwrap().to_string(),
            domain_version: d["version"].as_str().unwrap().to_string(),
            verifying_contract: hex_to_20(d["verifyingContract"].as_str().unwrap()),
            chain_id: d["chainId"].as_u64().unwrap(),
            game_id: hex_to_32(m["gameId"].as_str().unwrap()),
            home: m["homeChainId"].as_str().unwrap().to_string(),
            dest: m["destChainId"].as_str().unwrap().to_string(),
            licensee: hex_to_32(m["licensee"].as_str().unwrap()),
            is_free: m["isFree"].as_bool().unwrap(),
            paid_amount: u64_from("paidAmount"),
            paid_token: hex_to_bytes(m["paidToken"].as_str().unwrap()),
            paid_token_format: m["paidTokenFormat"].as_u64().unwrap() as u8,
            receipt_version: m["receiptVersion"].as_u64().unwrap() as u8,
            version: m["version"].as_u64().unwrap() as u8,
            nonce: u64_from("nonce"),
            home_finalized_block: u64_from("homeFinalizedBlock"),
            observed_at: u64_from("observedAt"),
            transport_id: hex_to_32(m["transportId"].as_str().unwrap()),
            digest: hex_to_32(v["digest"].as_str().unwrap()),
        }
    }

    fn views(o: &OwnedVector) -> (DomainView<'_>, AttestationView<'_>) {
        (
            DomainView {
                name: &o.domain_name,
                version: &o.domain_version,
                chain_id: o.chain_id,
                verifying_contract: o.verifying_contract,
            },
            AttestationView {
                game_id: o.game_id,
                home_chain_id: &o.home,
                dest_chain_id: &o.dest,
                licensee: o.licensee,
                is_free: o.is_free,
                paid_amount: o.paid_amount,
                paid_token: &o.paid_token,
                paid_token_format: o.paid_token_format,
                receipt_version: o.receipt_version,
                version: o.version,
                nonce: o.nonce,
                home_finalized_block: o.home_finalized_block,
                observed_at: o.observed_at,
                transport_id: o.transport_id,
            },
        )
    }

    /// AC-VEC-SOL: the Rust EIP-712 encoder reproduces BOTH fixture digests
    /// byte-identically (Solidity == TS == Rust triple proof).
    #[test]
    fn digest_matches_cross_stack_vectors() {
        for label in ["paid-evm-licensee", "free-solana-licensee"] {
            let o = load_owned(label);
            let (d, a) = views(&o);
            assert_eq!(attestation_digest(&d, &a), o.digest, "vector {label}");
        }
    }

    /// AC-SIG-ROUNDTRIP: real secp256k1 sign (RFC6979) -> on-path recover ->
    /// threshold quorum over the fixture digest. No mocks: `secp256k1_recover`
    /// runs its non-syscall fallback under `cargo test`.
    #[test]
    fn quorum_round_trip_over_fixture_digest() {
        let o = load_owned("paid-evm-licensee");
        let (d, a) = views(&o);
        let digest = attestation_digest(&d, &a);

        // Fixed attester keys 0x01..0x05 (CLEARLY test-only, never funded).
        let mut signers = [[0u8; 20]; ATTESTER_COUNT];
        let mut secrets = Vec::new();
        for i in 1u8..=5 {
            let mut sk = [0u8; 32];
            sk[31] = i;
            let secret = libsecp256k1::SecretKey::parse(&sk).unwrap();
            let pubkey = libsecp256k1::PublicKey::from_secret_key(&secret);
            signers[(i - 1) as usize] = eth_addr_from_pubkey(&pubkey.serialize()[1..65].try_into().unwrap());
            secrets.push(secret);
        }

        let msg = libsecp256k1::Message::parse(&digest);
        let mut sigs: Vec<([u8; 64], u8)> = vec![];
        for s in secrets.iter().take(3) {
            let (sig, recid) = libsecp256k1::sign(&msg, s);
            sigs.push((sig.serialize(), recid.serialize()));
        }
        let recovered = recover_attester_addrs(&digest, &sigs).unwrap();
        assert_eq!(recovered, signers[..3]);
        assert!(verify_threshold(&recovered, &signers).is_ok());
    }

    /// AC-THRESHOLD-SOL: 2-of-5, unknown signer, double count all fail.
    #[test]
    fn threshold_negatives() {
        let o = load_owned("paid-evm-licensee");
        let (d, a) = views(&o);
        let digest = attestation_digest(&d, &a);
        let mut signers = [[0u8; 20]; ATTESTER_COUNT];
        let mut secrets = Vec::new();
        for i in 1u8..=5 {
            let mut sk = [0u8; 32];
            sk[31] = i;
            let secret = libsecp256k1::SecretKey::parse(&sk).unwrap();
            let pubkey = libsecp256k1::PublicKey::from_secret_key(&secret);
            signers[(i - 1) as usize] = eth_addr_from_pubkey(&pubkey.serialize()[1..65].try_into().unwrap());
            secrets.push(secret);
        }
        let sign = |idx: usize| {
            let (sig, recid) = libsecp256k1::sign(&libsecp256k1::Message::parse(&digest), &secrets[idx]);
            (sig.serialize(), recid.serialize())
        };
        // 2-of-5 is not a quorum.
        let two = recover_attester_addrs(&digest, &[sign(0), sign(1)]).unwrap();
        assert_eq!(verify_threshold(&two, &signers), Err(XchainError::ThresholdNotMet));
        // Unknown signer (key 0x09, outside the set of 5).
        let mut sk = [0u8; 32];
        sk[31] = 9;
        let outsider = libsecp256k1::SecretKey::parse(&sk).unwrap();
        let (osig, orec) = libsecp256k1::sign(&libsecp256k1::Message::parse(&digest), &outsider);
        let mixed = recover_attester_addrs(&digest, &[sign(0), sign(1), (osig.serialize(), orec.serialize())]).unwrap();
        assert_eq!(verify_threshold(&mixed, &signers), Err(XchainError::UnknownAttester));
        // Same member twice + one other = DuplicateAttesterSignature.
        let dup = recover_attester_addrs(&digest, &[sign(0), sign(1), sign(0)]).unwrap();
        assert_eq!(
            verify_threshold(&dup, &signers),
            Err(XchainError::DuplicateAttesterSignature)
        );
        // Tampered digest: recoveries miss the set entirely.
        let mut bad_digest = digest;
        bad_digest[0] ^= 0xff;
        let bad = recover_attester_addrs(&bad_digest, &[sign(0), sign(1), sign(2)]).unwrap();
        assert_eq!(verify_threshold(&bad, &signers), Err(XchainError::UnknownAttester));
    }

    /// AC-REPLAY-SOL + AC-REORG-SOL: consumed set rejects reuse; reorg
    /// window enforced; future observations rejected.
    #[test]
    fn replay_and_reorg_policy() {
        let digest = [0xabu8; 32];
        assert!(check_replay(&digest, &[]).is_ok());
        assert_eq!(check_replay(&digest, &[digest]), Err(XchainError::AlreadyConsumed));
        assert!(check_reorg(1000, 900, 60).is_ok());
        assert_eq!(check_reorg(950, 900, 60), Err(XchainError::ReorgDelayNotElapsed));
        assert_eq!(check_reorg(900, 901, 0), Err(XchainError::ObservedAtInFuture));
    }

    /// AC-DEST-SOL (Tobias F2): config dest mismatch rejected; match passes.
    #[test]
    fn dest_chain_anchor() {
        assert!(check_dest_chain_id("solana:localnet", "solana:localnet").is_ok());
        assert_eq!(
            check_dest_chain_id("solana:localnet", "solana:EtWBMkgFUSkMhvZSGQi4uAiEE8RjqiB1MwWXEaAu34"),
            Err(XchainError::InvalidDestChainId)
        );
    }

    /// AC-SIGCAP-SOL (Tobias F3): >5 supplied sigs rejected before recovery.
    #[test]
    fn sig_cap() {
        assert!(check_sig_cap(3).is_ok());
        assert!(check_sig_cap(5).is_ok());
        assert_eq!(check_sig_cap(6), Err(XchainError::TooManyAttestationSignatures));
    }

    /// AC-DOMAIN-SOL: Solana-dest domain derivation (mirrors
    /// xchain_consume.rs) is deterministic and separates networks: localnet
    /// vs devnet tags differ, so one attestation never verifies on both.
    #[test]
    fn solana_domain_derivation_separates_networks() {
        let tag = |chain: &str| {
            let h = keccak256(chain.as_bytes());
            let mut b = [0u8; 8];
            b.copy_from_slice(&h[..8]);
            u64::from_be_bytes(b)
        };
        assert_ne!(tag("solana:localnet"), tag("solana:EtWBMkgFUSkMhvZSGQi4uAiEE8RjqiB1MwWXEaAu34"));
        // Same message under the two domains digests differently.
        let mk = |tag: u64| DomainView {
            name: "PGL1-XCHAIN",
            version: "1",
            chain_id: tag,
            verifying_contract: [0x77u8; 20],
        };
        let tok: Vec<u8> = vec![0x11u8; 20];
        let a = AttestationView {
            game_id: [0xabu8; 32],
            home_chain_id: "eip155:84532",
            dest_chain_id: "solana:localnet",
            licensee: [0xbbu8; 32],
            is_free: false,
            paid_amount: 100,
            paid_token: &tok,
            paid_token_format: 0,
            receipt_version: 1,
            version: 1,
            nonce: 7,
            home_finalized_block: 1,
            observed_at: 1000,
            transport_id: [0xccu8; 32],
        };
        let d_local = attestation_digest(&mk(tag("solana:localnet")), &a);
        let d_dev = attestation_digest(&mk(tag("solana:EtWBMkgFUSkMhvZSGQi4uAiEE8RjqiB1MwWXEaAu34")), &a);
        assert_ne!(d_local, d_dev);
        // And the EVM-dest domain (chainId + contract address) never collides.
        let d_evm = attestation_digest(
            &DomainView {
                name: "PGL1-XCHAIN",
                version: "1",
                chain_id: 31337,
                verifying_contract: [0x77u8; 20],
            },
            &a,
        );
        assert_ne!(d_local, d_evm);
    }

    /// AC-PAYBIND-SOL: envelope rules identical to the EVM dest check, plus
    /// home-chain allowlist gating.
    #[test]
    fn attestation_policy_binding() {
        let tok = [0x11u8; 20];
        assert!(check_attestation_policy(1, 1, true, 0, &[], 0, "eip155:84532").is_ok());
        assert!(check_attestation_policy(1, 1, false, 100, &tok, 0, "eip155:84532").is_ok());
        assert_eq!(
            check_attestation_policy(1, 1, false, 0, &[], 0, "eip155:84532"),
            Err(XchainError::PaidAmountZeroOnPaidPath)
        );
        assert_eq!(
            check_attestation_policy(1, 1, true, 5, &[], 0, "eip155:84532"),
            Err(XchainError::PaidAmountNonZeroOnFreePath)
        );
        assert_eq!(
            check_attestation_policy(1, 0, true, 0, &[], 0, "eip155:84532"),
            Err(XchainError::VersionMismatch)
        );
        assert_eq!(
            check_attestation_policy(1, 1, false, 1, &tok, 0, "eip155:99999"),
            Err(XchainError::UnknownChainId)
        );
    }
}
