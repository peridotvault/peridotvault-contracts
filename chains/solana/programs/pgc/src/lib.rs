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
    views::{GetGameStateView, GetLicenseView, GetMinterView, LicensePolicyView},
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
    views::{
        __cpi_client_accounts_get_game_state_view, __cpi_client_accounts_get_license_view,
        __cpi_client_accounts_get_minter_view,
        __client_accounts_get_game_state_view, __client_accounts_get_license_view,
        __client_accounts_get_minter_view,
    },
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

    pub fn get_publisher(ctx: Context<GetGameStateView>) -> Result<Pubkey> {
        instructions::views::get_publisher(ctx)
    }

    pub fn get_game_id(ctx: Context<GetGameStateView>) -> Result<String> {
        instructions::views::get_game_id(ctx)
    }

    pub fn get_metadata_uri(ctx: Context<GetGameStateView>) -> Result<String> {
        instructions::views::get_metadata_uri(ctx)
    }

    pub fn is_minter(ctx: Context<GetMinterView>) -> Result<bool> {
        instructions::views::is_minter(ctx)
    }

    pub fn get_license_policy(ctx: Context<GetLicenseView>) -> Result<LicensePolicyView> {
        instructions::views::get_license_policy(ctx)
    }

    pub fn has_license(ctx: Context<GetLicenseView>) -> Result<bool> {
        instructions::views::has_license(ctx)
    }

    pub fn can_access_game(ctx: Context<GetLicenseView>) -> Result<bool> {
        instructions::views::can_access_game(ctx)
    }
}
