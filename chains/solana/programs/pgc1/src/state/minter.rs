use anchor_lang::prelude::*;

#[account]
pub struct MinterAccount {
    pub game: Pubkey,
    pub account: Pubkey,
    pub is_authorized: bool,
    pub bump: u8,
}

impl MinterAccount {
    pub const SPACE: usize = 8 + 32 + 32 + 1 + 1;
}
