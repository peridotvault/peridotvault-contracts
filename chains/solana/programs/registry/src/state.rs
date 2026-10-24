use anchor_lang::prelude::*;

pub const MAX_GAME_ID_LEN: usize = 64;
pub const MAX_METADATA_URI_LEN: usize = 256;

#[account]
pub struct RegistryConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub pgl1_program: Pubkey,
    pub bump: u8,
}

impl RegistryConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 32 + 1;
}

#[account]
pub struct AcceptedPaymentToken {
    pub mint: Pubkey,
    pub active: bool,
    pub fee_amount: u64,
    pub bump: u8,
}

impl AcceptedPaymentToken {
    pub const SPACE: usize = 8 + 32 + 1 + 8 + 1;
}

#[account]
pub struct RegistryGame {
    pub game: Pubkey,
    pub game_id: String,
    pub registered_at: i64,
    pub status: GameStatus,
    pub bump: u8,
}

impl RegistryGame {
    pub const SPACE: usize = 8 + 32 + (4 + MAX_GAME_ID_LEN) + 8 + 1 + 1;
}

#[account]
pub struct PublishGrant {
    pub expired_at: Option<i64>,
    pub bump: u8,
}

impl PublishGrant {
    pub const SPACE: usize = 8 + 1 + 8 + 1;

    pub fn is_active_at(&self, now: i64) -> bool {
        match self.expired_at {
            None => true,
            Some(ts) => now <= ts,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Active,
    Suspended,
    Banned,
}

impl GameStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Suspended),
            2 => Some(Self::Banned),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Active => 0,
            Self::Suspended => 1,
            Self::Banned => 2,
        }
    }
}

/// M1 cross-chain entry (ANT-490, spec §3.2). Q2 answer: `pending` lives
/// INLINE on the record — single-read path for the attester (NFR-6), same
/// choice as the EVM `XEntry.pending`.
pub const MAX_CHAIN_ID_LEN: usize = 64;
pub const MAX_PROGRAM_ID_LEN: usize = 32;
pub const MAX_LOCAL_ID_LEN: usize = 128;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum XchainStatus {
    Proposed,
    Active,
    Deprecated,
    Frozen,
}

impl XchainStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Proposed),
            1 => Some(Self::Active),
            2 => Some(Self::Deprecated),
            3 => Some(Self::Frozen),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Proposed => 0,
            Self::Active => 1,
            Self::Deprecated => 2,
            Self::Frozen => 3,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum XchainIdFormat {
    Evm20,
    Solana32,
}

impl XchainIdFormat {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Evm20),
            1 => Some(Self::Solana32),
            _ => None,
        }
    }

    pub fn byte_len(&self) -> usize {
        match self {
            Self::Evm20 => 20,
            Self::Solana32 => 32,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct XchainPendingUpdate {
    pub proposed_program_id: Vec<u8>,
    pub proposed_format: XchainIdFormat,
    pub proposed_at: i64,
    pub effective_at: i64,
    pub timelock_delay: i64,
    pub proposer: Vec<u8>,
}

#[account]
pub struct XchainGameEntry {
    pub chain_id: String,
    pub program_id: Vec<u8>,
    pub format: XchainIdFormat,
    /// M1 schema version (M0 M5 lineage, CURRENT_VERSION = 1).
    pub version: u8,
    pub status: XchainStatus,
    pub game_id: [u8; 32],
    pub registered_at: i64,
    /// Inline timelock state (§3.3); None = no update in flight.
    pub pending: Option<XchainPendingUpdate>,
}

impl XchainGameEntry {
    pub const SPACE: usize = 8
        + (4 + MAX_CHAIN_ID_LEN)
        + (4 + MAX_PROGRAM_ID_LEN)
        + 1
        + 1
        + 1
        + 32
        + 8
        + (1 + (4 + MAX_PROGRAM_ID_LEN) + 1 + 8 + 8 + 8 + (4 + MAX_PROGRAM_ID_LEN));
}
