use anchor_lang::prelude::*;

#[account]
pub struct PgcGameAccount {
    pub game_id: String,     // Max 32 chars
    pub publisher: Pubkey,   // 32 bytes
    pub metadata_uri: String, // Max 200 chars
    pub created_at: i64,     // 8 bytes
    pub bump: u8,            // 1 byte
}

impl PgcGameAccount {
    pub const SPACE: usize = 8 + 36 + 32 + 204 + 8 + 1;
}
