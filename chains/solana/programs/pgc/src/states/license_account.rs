use anchor_lang::prelude::*;

#[account]
pub struct LicenseAccount {
    pub bump: u8,
    pub game: Pubkey,
    pub user: Pubkey,
    pub issued_at: i64,
    pub expires_at: i64, // 0 = permanent
    pub badge_minted: bool,
}

impl LicenseAccount {
    pub const SPACE: usize = 8 + // discriminator
        1 + // bump
        32 + // game
        32 + // user
        8 + // issued_at
        8 + // expires_at
        1; // badge_minted

    pub fn is_valid(&self, now: i64) -> bool {
        self.expires_at == 0 || self.expires_at > now
    }

    pub fn is_permanent(&self) -> bool {
        self.expires_at == 0
    }
}
