use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP");

#[program]
pub mod pgl1 {
    use super::*;

    pub fn initialize_pgl(
        ctx: Context<InitializePgl>,
        treasury: Pubkey,
        create_game_fee_lamports: u64,
    ) -> Result<()> {
        initialize_pgl::handler(ctx, treasury, create_game_fee_lamports)
    }

    pub fn set_create_game_fee(
        ctx: Context<SetCreateGameFee>,
        create_game_fee_lamports: u64,
    ) -> Result<()> {
        set_create_game_fee::handler(ctx, create_game_fee_lamports)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        set_treasury::handler(ctx, treasury)
    }

    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        set_authority::handler(ctx, new_authority)
    }

    pub fn add_authorized_actor(ctx: Context<AddAuthorizedActor>) -> Result<()> {
        add_authorized_actor::handler(ctx)
    }

    pub fn deactivate_authorized_actor(ctx: Context<DeactivateAuthorizedActor>) -> Result<()> {
        deactivate_authorized_actor::handler(ctx)
    }

    pub fn close_authorized_actor(ctx: Context<CloseAuthorizedActor>) -> Result<()> {
        close_authorized_actor::handler(ctx)
    }

    pub fn close_creator_state(ctx: Context<CloseCreatorState>) -> Result<()> {
        close_creator_state::handler(ctx)
    }

    pub fn create_game(
        ctx: Context<CreateGame>,
        game_id: String,
        metadata_uri: String,
    ) -> Result<()> {
        create_game::handler(ctx, game_id, metadata_uri)
    }

    pub fn set_publisher(ctx: Context<SetPublisher>, new_publisher: Pubkey) -> Result<()> {
        set_publisher::handler(ctx, new_publisher)
    }

    pub fn set_metadata_uri(ctx: Context<SetMetadataUri>, metadata_uri: String) -> Result<()> {
        set_metadata_uri::handler(ctx, metadata_uri)
    }

    pub fn mint_license(ctx: Context<MintLicense>, expires_at: Option<i64>) -> Result<()> {
        mint_license::handler(ctx, expires_at)
    }

    pub fn renew_license(ctx: Context<RenewLicense>, expires_at: i64) -> Result<()> {
        renew_license::handler(ctx, expires_at)
    }
}
