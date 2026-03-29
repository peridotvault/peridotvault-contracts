use anchor_lang::prelude::*;

#[event]
pub struct GameCreated {
    pub game_id: String,
    pub publisher: Pubkey,
}

#[event]
pub struct LicenseIssued {
    pub owner: Pubkey,
    pub game: Pubkey,
    pub expires_at: i64,
}

#[event]
pub struct MinterUpdated {
    pub game: Pubkey,
    pub minter: Pubkey,
    pub is_authorized: bool,
}

#[event]
pub struct LicenseRevoked {
    pub owner: Pubkey,
    pub game: Pubkey,
}

#[event]
pub struct MetadataUpdated {
    pub game: Pubkey,
    pub new_uri: String,
}
#[event]
pub struct PublisherUpdated {
    pub game: Pubkey,
    pub old_publisher: Pubkey,
    pub new_publisher: Pubkey,
}
