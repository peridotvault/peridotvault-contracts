use anchor_lang::prelude::*;

#[event]
pub struct RegistryInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub pgl1_program: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub old_treasury: Pubkey,
    pub new_treasury: Pubkey,
}

#[event]
pub struct PaymentTokenAdded {
    pub mint: Pubkey,
    pub fee_amount: u64,
}

#[event]
pub struct PaymentTokenUpdated {
    pub mint: Pubkey,
    pub old_active: bool,
    pub new_active: bool,
    pub old_fee_amount: u64,
    pub new_fee_amount: u64,
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
    pub old_expired_at: Option<i64>,
    pub new_expired_at: Option<i64>,
}

#[event]
pub struct GameRegistered {
    pub game: Pubkey,
    pub game_id: String,
    pub publisher: Pubkey,
    pub status: u8,
    pub registered_at: i64,
}

#[event]
pub struct GameStatusUpdated {
    pub game: Pubkey,
    pub old_status: u8,
    pub new_status: u8,
    pub authority: Pubkey,
}

#[event]
pub struct GameClosed {
    pub game: Pubkey,
    pub game_id: String,
    pub closed_by: Pubkey,
}
