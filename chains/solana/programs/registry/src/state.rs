use anchor_lang::prelude::*;

#[account]
pub struct RegistryGameAccount {
    pub game_id: String,
    pub publisher: Pubkey,
    pub pgc_pid: Pubkey,   // Program Implementation ID
    pub pgc_pda: Pubkey,   // Game State Instance PDA
    pub active: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl RegistryGameAccount {
    pub const SPACE: usize = 8 + 4 + 32 + 32 + 32 + 32 + 1 + 8 + 1;
}

#[account]
pub struct RegistryConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub registration_fee: u64,
    pub registration_currency: Pubkey, // System Program for SOL
    pub bump: u8,
}

impl RegistryConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 32 + 1;
}
