use anchor_lang::prelude::*;

#[event]
pub struct GameCreated {
    pub game_id: String,
    pub publisher: Pubkey,
    pub metadata_uri: String,
}

#[event]
pub struct LicenseIssued {
    pub owner: Pubkey,
    pub game: Pubkey,
    pub expires_at: i64,
}

#[event]
pub struct LicenseRevoked {
    pub owner: Pubkey,
    pub game: Pubkey,
}
