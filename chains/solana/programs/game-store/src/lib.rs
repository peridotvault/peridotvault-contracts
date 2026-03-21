use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod states;

pub use instructions::{
    buy_game::BuyGame,
    buy_game_native_sol::BuyGameNativeSol,
    initialize::Initialize,
    set_discount::SetDiscount,
    set_governance::SetGovernance,
    set_platform_fee::SetPlatformFee,
    set_price::SetPrice,
    set_treasury::SetTreasury,
    withdraw::Withdraw,
    withdraw_sol::WithdrawSol,
};
pub use states::PriceConfig;
use instructions::{
    buy_game::__client_accounts_buy_game,
    buy_game_native_sol::__client_accounts_buy_game_native_sol,
    initialize::__client_accounts_initialize,
    set_discount::__client_accounts_set_discount,
    set_governance::__client_accounts_set_governance,
    set_platform_fee::__client_accounts_set_platform_fee,
    set_price::__client_accounts_set_price,
    set_treasury::__client_accounts_set_treasury,
    withdraw::__client_accounts_withdraw,
    withdraw_sol::__client_accounts_withdraw_sol,
};

declare_id!("DSiyompbYR2k2GsS69FWkvE9N3vf32Da4JNqZKYvn2Pp");

#[program]
pub mod game_store {
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

    pub fn buy_game_native_sol(
        ctx: Context<BuyGameNativeSol>,
        game_id: String,
    ) -> Result<()> {
        instructions::buy_game_native_sol::handler(ctx, game_id)
    }

    pub fn set_platform_fee(ctx: Context<SetPlatformFee>, fee_bps: u16) -> Result<()> {
        instructions::set_platform_fee::handler(ctx, fee_bps)
    }

    pub fn withdraw(ctx: Context<Withdraw>, token: Pubkey) -> Result<()> {
        instructions::withdraw::handler(ctx, token)
    }

    pub fn withdraw_sol(ctx: Context<WithdrawSol>) -> Result<()> {
        instructions::withdraw_sol::handler(ctx)
    }

    pub fn set_governance(ctx: Context<SetGovernance>, governance: Pubkey) -> Result<()> {
        instructions::set_governance::handler(ctx, governance)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        instructions::set_treasury::handler(ctx, treasury)
    }
}
