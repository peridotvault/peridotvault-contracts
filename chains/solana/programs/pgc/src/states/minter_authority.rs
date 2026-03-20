use anchor_lang::prelude::*;

#[account]
pub struct MinterAuthority {
    pub bump: u8,
    pub game: Pubkey,
    pub account: Pubkey,
    pub is_authorized: bool,
}

impl MinterAuthority {
    pub const SPACE: usize = 8 + // discriminator
        1 + // bump
        32 + // game
        32 + // account
        1; // is_authorized
}
