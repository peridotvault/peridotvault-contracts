use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod states;

pub use instructions::{
    buy_game::BuyGame,
    initialize::Initialize,
    set_discount::SetDiscount,
    set_governance::SetGovernance,
    set_platform_fee::SetPlatformFee,
    set_price::SetPrice,
    set_treasury::SetTreasury,
    views::GetStoreView,
    withdraw::Withdraw,
};
pub use states::PriceConfig;
use instructions::{
    buy_game::__client_accounts_buy_game,
    initialize::__client_accounts_initialize,
    set_discount::__client_accounts_set_discount,
    set_governance::__client_accounts_set_governance,
    set_platform_fee::__client_accounts_set_platform_fee,
    set_price::__client_accounts_set_price,
    set_treasury::__client_accounts_set_treasury,
    views::__client_accounts_get_store_view,
    withdraw::__client_accounts_withdraw,
};

declare_id!("DSiyompbYR2k2GsS69FWkvE9N3vf32Da4JNqZKYvn2Pp");

#[program]
pub mod solana {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        governance: Pubkey,
        treasury: Pubkey,
        registry: Pubkey,
        platform_fee_bps: u16,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, governance, treasury, registry, platform_fee_bps)
    }

    pub fn set_price(
        ctx: Context<SetPrice>,
        game_id: String,
        price: u64,
        currency: Pubkey,
    ) -> Result<()> {
        instructions::set_price::handler(ctx, game_id, price, currency)
    }

    pub fn set_discount(
        ctx: Context<SetDiscount>,
        game_id: String,
        discount_bps: u16,
    ) -> Result<()> {
        instructions::set_discount::handler(ctx, game_id, discount_bps)
    }

    pub fn buy_game(ctx: Context<BuyGame>, game_id: String) -> Result<()> {
        instructions::buy_game::handler(ctx, game_id)
    }

    pub fn set_platform_fee(ctx: Context<SetPlatformFee>, fee_bps: u16) -> Result<()> {
        instructions::set_platform_fee::handler(ctx, fee_bps)
    }

    pub fn withdraw(ctx: Context<Withdraw>, token: Pubkey) -> Result<()> {
        instructions::withdraw::handler(ctx, token)
    }

    pub fn set_governance(ctx: Context<SetGovernance>, governance: Pubkey) -> Result<()> {
        instructions::set_governance::handler(ctx, governance)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        instructions::set_treasury::handler(ctx, treasury)
    }

    pub fn get_price_config(ctx: Context<GetStoreView>, game_id: String) -> Result<PriceConfig> {
        instructions::views::get_price_config(ctx, game_id)
    }

    pub fn get_publisher_balance(
        ctx: Context<GetStoreView>,
        publisher: Pubkey,
        token: Pubkey,
    ) -> Result<u64> {
        instructions::views::get_publisher_balance(ctx, publisher, token)
    }

    pub fn get_platform_fee(ctx: Context<GetStoreView>) -> Result<u16> {
        instructions::views::get_platform_fee(ctx)
    }

    pub fn get_treasury(ctx: Context<GetStoreView>) -> Result<Pubkey> {
        instructions::views::get_treasury(ctx)
    }

    pub fn get_governance(ctx: Context<GetStoreView>) -> Result<Pubkey> {
        instructions::views::get_governance(ctx)
    }

    pub fn get_registry(ctx: Context<GetStoreView>) -> Result<Pubkey> {
        instructions::views::get_registry(ctx)
    }

    pub fn get_final_price(ctx: Context<GetStoreView>, game_id: String) -> Result<u64> {
        instructions::views::get_final_price(ctx, game_id)
    }
}
