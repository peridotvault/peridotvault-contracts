use anchor_lang::prelude::*;

#[event]
pub struct Initialized {
    pub game: Pubkey,
    pub publisher: Pubkey,
    pub initial_minter: Pubkey,
    pub mint: Pubkey,
    pub game_id: String,
    pub metadata_uri: String,
}

#[event]
pub struct LicenseMinted {
    pub game: Pubkey,
    pub user: Pubkey,
    pub issued_at: i64,
    pub expires_at: i64,
    pub minter: Pubkey,
    pub badge_minted: bool,
}

#[event]
pub struct MinterUpdated {
    pub game: Pubkey,
    pub account: Pubkey,
    pub is_authorized: bool,
}

#[event]
pub struct PublisherUpdated {
    pub game: Pubkey,
    pub old_publisher: Pubkey,
    pub new_publisher: Pubkey,
}

#[event]
pub struct MetadataUriUpdated {
    pub game: Pubkey,
    pub metadata_uri: String,
}
