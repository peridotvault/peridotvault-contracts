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
