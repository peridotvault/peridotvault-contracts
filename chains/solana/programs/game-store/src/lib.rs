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
    withdraw::Withdraw,
};
pub use states::PriceConfig;
#[allow(unused_imports)]
use instructions::{
    buy_game::__cpi_client_accounts_buy_game,
    buy_game::__client_accounts_buy_game,
    initialize::__cpi_client_accounts_initialize,
    initialize::__client_accounts_initialize,
    set_discount::__cpi_client_accounts_set_discount,
    set_discount::__client_accounts_set_discount,
    set_governance::__cpi_client_accounts_set_governance,
    set_governance::__client_accounts_set_governance,
    set_platform_fee::__cpi_client_accounts_set_platform_fee,
    set_platform_fee::__client_accounts_set_platform_fee,
    set_price::__cpi_client_accounts_set_price,
    set_price::__client_accounts_set_price,
    set_treasury::__cpi_client_accounts_set_treasury,
    set_treasury::__client_accounts_set_treasury,
    withdraw::__cpi_client_accounts_withdraw,
    withdraw::__client_accounts_withdraw,
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
}
