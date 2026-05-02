use anchor_lang::prelude::*;

#[event]
pub struct PglInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub create_game_fee_lamports: u64,
}

#[event]
pub struct CreateGameFeeUpdated {
    pub old_fee: u64,
    pub new_fee: u64,
}

#[event]
pub struct TreasuryUpdated {
    pub authority: Pubkey,
    pub old_treasury: Pubkey,
    pub new_treasury: Pubkey,
}

#[event]
pub struct AuthorityUpdated {
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
pub struct AuthorizedActorAdded {
    pub actor: Pubkey,
}

#[event]
pub struct AuthorizedActorDeactivated {
    pub actor: Pubkey,
}

#[event]
pub struct AuthorizedActorClosed {
    pub actor: Pubkey,
}

#[event]
pub struct CreatorStateClosed {
    pub creator: Pubkey,
}

#[event]
pub struct GameCreated {
    pub game: Pubkey,
    pub creator: Pubkey,
    pub publisher: Pubkey,
    pub nonce: u64,
    pub game_id: String,
    pub metadata_uri: String,
    pub created_at: i64,
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
    pub publisher: Pubkey,
    pub old_uri: String,
    pub new_uri: String,
}

#[event]
pub struct LicenseMinted {
    pub license: Pubkey,
    pub actor: Pubkey,
    pub holder: Pubkey,
    pub game: Pubkey,
    pub issued_at: i64,
    pub expires_at: Option<i64>,
}

#[event]
pub struct LicenseRenewed {
    pub license: Pubkey,
    pub actor: Pubkey,
    pub holder: Pubkey,
    pub game: Pubkey,
    pub old_expires_at: Option<i64>,
    pub new_expires_at: i64,
}
