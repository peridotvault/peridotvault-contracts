use anchor_lang::prelude::*;

#[account]
pub struct PgcConfig {
    pub authority: Pubkey,
    pub version: u8,
}

impl PgcConfig {
    pub const SPACE: usize = 8 + 32 + 1; // 41 bytes.
}
