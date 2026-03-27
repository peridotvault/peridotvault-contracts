use anchor_lang::prelude::*;

#[account]
pub struct RegistryGameAccount {
    pub game_id: String,
    pub publisher: Pubkey,
    pub pgc_program: Pubkey,
    pub pgc_game: Pubkey,
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
    pub bump: u8,
}

impl RegistryConfig {
    pub const SPACE: usize = 8 + 32 + 1;
}
