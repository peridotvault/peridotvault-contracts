use anchor_lang::prelude::*;

#[account]
pub struct FactoryState {
    pub bump: u8,
    pub registry: Pubkey,
    pub game_store: Pubkey,
    pub governance: Pubkey,
}

impl FactoryState {
    pub const SPACE: usize = 8 + 1 + 32 + 32 + 32;
}
