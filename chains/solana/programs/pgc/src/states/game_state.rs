use crate::constants::*;
use anchor_lang::prelude::*;

#[account]
pub struct GameState {
    pub bump: u8,
    pub authority_bump: u8,
    pub mint: Pubkey,
    pub game_id: String,
    pub publisher: Pubkey,
    pub metadata_uri: String,
}

impl GameState {
    pub const SPACE: usize = 8 + // discriminator
        1 + // bump
        1 + // authority_bump
        32 + // mint
        4 + MAX_GAME_ID_LEN +
        32 + // publisher
        4 + MAX_METADATA_URI_LEN;
}
