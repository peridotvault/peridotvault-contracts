use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

pub use instructions::*;

declare_id!("8pgmtQDVpMX4FHmoCmWJCoB94RY56GKWUzo8f8e1Xfpo");

#[program]
pub mod registry {
    use super::*;

    pub fn initialize_registry(ctx: Context<InitializeRegistry>, treasury: Pubkey) -> Result<()> {
        instructions::initialize_registry::handler(ctx, treasury)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        instructions::set_treasury::handler(ctx, treasury)
    }

    pub fn add_payment_token(ctx: Context<AddPaymentToken>, fee_amount: u64) -> Result<()> {
        instructions::add_payment_token::handler(ctx, fee_amount)
    }

    pub fn update_payment_token(
        ctx: Context<UpdatePaymentToken>,
        active: bool,
        fee_amount: u64,
    ) -> Result<()> {
        instructions::update_payment_token::handler(ctx, active, fee_amount)
    }

    pub fn remove_payment_token(ctx: Context<RemovePaymentToken>) -> Result<()> {
        instructions::remove_payment_token::handler(ctx)
    }

    pub fn create_publish_grant(ctx: Context<CreatePublishGrant>, expired_at: Option<i64>) -> Result<()> {
        instructions::set_publish_grant::create_handler(ctx, expired_at)
    }

    pub fn update_publish_grant(ctx: Context<UpdatePublishGrant>, expired_at: Option<i64>) -> Result<()> {
        instructions::set_publish_grant::update_handler(ctx, expired_at)
    }

    pub fn create_game_and_register<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateGameAndRegister<'info>>,
        game_id: String,
        metadata_uri: String,
        base_price: Option<u64>,
        mint_token: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_game_and_register::handler(ctx, game_id, metadata_uri, base_price, mint_token)
    }

    pub fn update_game_status(ctx: Context<UpdateGameStatus>, status: u8) -> Result<()> {
        instructions::update_game_status::handler(ctx, status)
    }

    pub fn close_registry_game(ctx: Context<CloseRegistryGame>) -> Result<()> {
        instructions::close_registry_game::handler(ctx)
    }
}
