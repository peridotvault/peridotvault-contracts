use anchor_lang::prelude::*;

#[account]
pub struct LicenseAccount {
    pub owner: Pubkey,
    pub game: Pubkey,
    pub issued_at: i64,
    pub expires_at: i64,
    pub bump: u8,
}

impl LicenseAccount {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 8 + 1; // 89 bytes. Well under 200.
}
