use anchor_lang::prelude::*;

pub const PGL_CONFIG_SEED: &[u8] = b"pgl_config";
pub const AUTHORIZED_ACTOR_SEED: &[u8] = b"authorized_actor";
pub const CREATOR_STATE_SEED: &[u8] = b"creator_state";
pub const GAME_SEED: &[u8] = b"game";
pub const LICENSE_SEED: &[u8] = b"license";

pub const MAX_GAME_ID_LEN: usize = 64;
pub const MAX_METADATA_URI_LEN: usize = 256;

#[account]
pub struct PglConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub create_game_fee_lamports: u64,
    pub bump: u8,
}

impl PglConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 1;
}

#[account]
pub struct AuthorizedActor {
    pub actor: Pubkey,
    pub active: bool,
    pub bump: u8,
}

impl AuthorizedActor {
    pub const SPACE: usize = 8 + 32 + 1 + 1;
}

#[account]
pub struct CreatorState {
    pub creator: Pubkey,
    pub next_nonce: u64,
    pub bump: u8,
}

impl CreatorState {
    pub const SPACE: usize = 8 + 32 + 8 + 1;
}

#[account]
pub struct Game {
    pub creator: Pubkey,
    pub nonce: u64,
    pub publisher: Pubkey,
    pub game_id: String,
    pub metadata_uri: String,
    pub created_at: i64,
    pub bump: u8,
}

impl Game {
    pub const SPACE: usize = 8
        + 32
        + 8
        + 32
        + 4 + MAX_GAME_ID_LEN
        + 4 + MAX_METADATA_URI_LEN
        + 8
        + 1;
}

#[account]
pub struct License {
    pub holder: Pubkey,
    pub game: Pubkey,
    pub issued_at: i64,
    pub expires_at: Option<i64>,
    pub bump: u8,
}

impl License {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + (1 + 8) + 1;
}
