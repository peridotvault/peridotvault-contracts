use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("6gTd8TQ9NiC7yxBfGWBzH1aWdk77fg779nUJhYTrEsPd");

#[program]
pub mod peridotvault_store {
    use super::*;

    pub fn initialize_store(
        ctx: Context<InitializeStore>,
        treasury: Pubkey,
        platform_fee_bps: u16,
        default_referral_bps: u16,
        max_referral_bps: u16,
        store_actor: Pubkey,
    ) -> Result<()> {
        initialize_store::handler(
            ctx,
            treasury,
            platform_fee_bps,
            default_referral_bps,
            max_referral_bps,
            store_actor,
        )
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        set_treasury::handler(ctx, treasury)
    }

    pub fn set_platform_fee(ctx: Context<SetPlatformFee>, platform_fee_bps: u16) -> Result<()> {
        set_platform_fee::handler(ctx, platform_fee_bps)
    }

    pub fn set_default_referral(
        ctx: Context<SetDefaultReferral>,
        default_referral_bps: u16,
    ) -> Result<()> {
        set_default_referral::handler(ctx, default_referral_bps)
    }

    pub fn set_max_referral(ctx: Context<SetMaxReferral>, max_referral_bps: u16) -> Result<()> {
        set_max_referral::handler(ctx, max_referral_bps)
    }

    pub fn add_authorized_source_program(ctx: Context<AddAuthorizedSourceProgram>) -> Result<()> {
        add_authorized_source_program::handler(ctx)
    }

    pub fn update_authorized_source_program(
        ctx: Context<UpdateAuthorizedSourceProgram>,
        active: bool,
    ) -> Result<()> {
        update_authorized_source_program::handler(ctx, active)
    }

    pub fn add_authorized_registry_program(
        ctx: Context<AddAuthorizedRegistryProgram>,
    ) -> Result<()> {
        add_authorized_registry_program::handler(ctx)
    }

    pub fn update_authorized_registry_program(
        ctx: Context<UpdateAuthorizedRegistryProgram>,
        active: bool,
    ) -> Result<()> {
        update_authorized_registry_program::handler(ctx, active)
    }

    pub fn add_payment_token(ctx: Context<AddPaymentToken>) -> Result<()> {
        add_payment_token::handler(ctx)
    }

    pub fn update_payment_token(ctx: Context<UpdatePaymentToken>, active: bool) -> Result<()> {
        update_payment_token::handler(ctx, active)
    }

    pub fn init_game_store_config(ctx: Context<InitGameStoreConfig>, active: bool) -> Result<()> {
        init_game_store_config::handler(ctx, active)
    }

    pub fn set_game_store_active(ctx: Context<SetGameStoreActive>, active: bool) -> Result<()> {
        set_game_store_active::handler(ctx, active)
    }

    pub fn set_game_payment_option(
        ctx: Context<SetGamePaymentOption>,
        base_price: u64,
        active: bool,
    ) -> Result<()> {
        set_game_payment_option::handler(ctx, base_price, active)
    }

    pub fn remove_game_payment_option(ctx: Context<RemoveGamePaymentOption>) -> Result<()> {
        remove_game_payment_option::handler(ctx)
    }

    pub fn set_discount(
        ctx: Context<SetDiscount>,
        discount_bps: Option<u16>,
        discount_starts_at: Option<i64>,
        discount_expires_at: Option<i64>,
    ) -> Result<()> {
        set_discount::handler(ctx, discount_bps, discount_starts_at, discount_expires_at)
    }

    pub fn clear_discount(ctx: Context<ClearDiscount>) -> Result<()> {
        clear_discount::handler(ctx)
    }

    pub fn set_referral_bps(ctx: Context<SetReferralBps>, referral_bps: Option<u16>) -> Result<()> {
        set_referral_bps::handler(ctx, referral_bps)
    }

    pub fn set_store_actor(ctx: Context<SetStoreActor>, new_store_actor: Pubkey) -> Result<()> {
        set_store_actor::handler(ctx, new_store_actor)
    }

    pub fn buy_game(
        ctx: Context<BuyGame>,
        paid_amount: u64,
        referrer: Option<Pubkey>,
    ) -> Result<()> {
        buy_game::handler(ctx, paid_amount, referrer)
    }
}
