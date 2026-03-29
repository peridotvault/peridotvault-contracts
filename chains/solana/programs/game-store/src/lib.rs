use anchor_lang::prelude::*;

pub mod state;
pub mod errors;
pub mod instructions;

pub use state::*;
pub use errors::*;
pub use instructions::*;

declare_id!("6gTd8TQ9NiC7yxBfGWBzH1aWdk77fg779nUJhYTrEsPd");

#[program]
pub mod game_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, platform_fee_bps: u16, treasury: Pubkey) -> Result<()> {
        initialize_handler(ctx, platform_fee_bps, treasury)
    }

    pub fn set_price(ctx: Context<SetPrice>, price: u64, currency: Pubkey) -> Result<()> {
        set_price_handler(ctx, price, currency)
    }

    pub fn buy_game(ctx: Context<BuyGame>) -> Result<()> {
        buy_game_handler(ctx)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        withdraw_handler(ctx)
    }

    pub fn set_platform_fee(ctx: Context<SetPlatformFee>, bps: u16) -> Result<()> {
        set_platform_fee_handler(ctx, bps)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        set_treasury_handler(ctx, treasury)
    }

    pub fn set_affiliate(ctx: Context<SetAffiliate>, affiliate: Pubkey, bps: u16) -> Result<()> {
        set_affiliate_handler(ctx, affiliate, bps)
    }

    pub fn set_subscription(ctx: Context<SetSubscription>, duration: i64, price: u64) -> Result<()> {
        set_subscription_handler(ctx, duration, price)
    }
}
