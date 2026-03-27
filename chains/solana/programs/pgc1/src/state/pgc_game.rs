use anchor_lang::prelude::*;

pub const MAX_GAME_ID_LEN: usize = 32;
pub const MAX_METADATA_URI_LEN: usize = 90;

#[account]
pub struct PgcGameAccount {
    pub game_id: String,
    pub publisher: Pubkey,
    pub metadata_uri: String,
    pub mint: Option<Pubkey>,
    pub bump: u8,
}

impl PgcGameAccount {
    pub const SPACE: usize = 8 + 4 + MAX_GAME_ID_LEN + 32 + 4 + MAX_METADATA_URI_LEN + 33 + 1; 
    // 8 + 4 + 32 + 32 + 4 + 90 + 33 + 1 = 204. Close enough.
}
