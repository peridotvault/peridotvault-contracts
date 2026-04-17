use anchor_lang::prelude::*;

#[account]
pub struct PgcConfig {
    pub authority: Pubkey,
    pub authorized_store: Pubkey,
    pub bump: u8,
}

impl PgcConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 1;
}
