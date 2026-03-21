use anchor_lang::prelude::*;

#[event]
pub struct FactoryInitialized {
    pub governance: Pubkey,
    pub registry: Pubkey,
    pub game_store: Pubkey,
}

#[event]
pub struct GameCreated {
    pub game_id: String,
    pub metadata_uri: String,
    pub publisher: Pubkey,
    pub game: Pubkey,
    pub mint: Pubkey,
}

#[event]
pub struct RegistryUpdated {
    pub old_registry: Pubkey,
    pub new_registry: Pubkey,
}

#[event]
pub struct GameStoreUpdated {
    pub old_game_store: Pubkey,
    pub new_game_store: Pubkey,
}

#[event]
pub struct GovernanceUpdated {
    pub old_governance: Pubkey,
    pub new_governance: Pubkey,
}
