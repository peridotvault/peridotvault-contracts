use anchor_lang::prelude::*;

#[event]
pub struct RegistryInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub pgl1_program: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub treasury: Pubkey,
}

#[event]
pub struct PaymentTokenAdded {
    pub mint: Pubkey,
    pub fee_amount: u64,
}

#[event]
pub struct PaymentTokenUpdated {
    pub mint: Pubkey,
    pub active: bool,
    pub fee_amount: u64,
}

#[event]
pub struct PaymentTokenRemoved {
    pub mint: Pubkey,
}

#[event]
pub struct PublishGrantCreated {
    pub publisher: Pubkey,
    pub expired_at: Option<i64>,
}

#[event]
pub struct PublishGrantUpdated {
    pub publisher: Pubkey,
    pub expired_at: Option<i64>,
}

#[event]
pub struct GameRegistered {
    pub game: Pubkey,
    pub game_id: String,
    pub status: u8,
}

#[event]
pub struct GameStatusUpdated {
    pub game: Pubkey,
    pub status: u8,
}

#[event]
pub struct GameClosed {
    pub game: Pubkey,
    pub game_id: String,
}
