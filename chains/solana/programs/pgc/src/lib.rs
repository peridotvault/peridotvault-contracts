use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod states;

pub use instructions::{
    initialize::Initialize,
    mint_license::MintLicense,
    set_metadata_uri::SetMetadataUri,
    set_minter::SetMinter,
    set_publisher::SetPublisher,
};
#[allow(unused_imports)]
use instructions::{
    initialize::__cpi_client_accounts_initialize,
    initialize::__client_accounts_initialize,
    mint_license::__cpi_client_accounts_mint_license,
    mint_license::__client_accounts_mint_license,
    set_metadata_uri::__cpi_client_accounts_set_metadata_uri,
    set_metadata_uri::__client_accounts_set_metadata_uri,
    set_minter::__cpi_client_accounts_set_minter,
    set_minter::__client_accounts_set_minter,
    set_publisher::__cpi_client_accounts_set_publisher,
    set_publisher::__client_accounts_set_publisher,
};

declare_id!("BDqzDEUTfzskChktZwNsceHj3Vnr7g3322JgPKrMqsip");

#[program]
pub mod pgc1 {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        game_id: String,
        publisher: Pubkey,
        metadata_uri: String,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, game_id, publisher, metadata_uri)
    }

    pub fn mint_license(ctx: Context<MintLicense>, expires_at: i64) -> Result<()> {
        instructions::mint_license::handler(ctx, expires_at)
    }

    pub fn set_minter(ctx: Context<SetMinter>, is_authorized: bool) -> Result<()> {
        instructions::set_minter::handler(ctx, is_authorized)
    }

    pub fn set_publisher(ctx: Context<SetPublisher>) -> Result<()> {
        instructions::set_publisher::handler(ctx)
    }

    pub fn set_metadata_uri(ctx: Context<SetMetadataUri>, metadata_uri: String) -> Result<()> {
        instructions::set_metadata_uri::handler(ctx, metadata_uri)
    }
}
