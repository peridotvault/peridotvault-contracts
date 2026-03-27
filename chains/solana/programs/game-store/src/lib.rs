use anchor_lang::prelude::*;
use pgc1::PgcGameAccount;

pub mod constants;
pub mod errors;
pub mod events;
pub mod state;
pub mod instructions;

pub use constants::*;
pub use errors::*;
pub use events::*;

pub use instructions::*;

declare_id!("7Z9MDRw8oALQyZQGF6LDh9G1t9mQvN3pFsuXzfb31MnS");

#[program]
pub mod game_store {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        governance: Pubkey,
        treasury: Pubkey,
        platform_fee_bps: u16,
    ) -> Result<()> {
        crate::instructions::initialize_handler(ctx, governance, treasury, platform_fee_bps)
    }

    pub fn set_price(
        ctx: Context<SetPrice>,
        price: u64,
        currency: Pubkey,
    ) -> Result<()> {
        crate::instructions::set_price_handler(ctx, price, currency)
    }

    pub fn set_affiliate(
        ctx: Context<SetAffiliate>,
        share_bps: u16,
    ) -> Result<()> {
        crate::instructions::set_affiliate_handler(ctx, share_bps)
    }

    pub fn set_subscription(
        ctx: Context<SetSubscription>,
        price: u64,
        duration: i64,
        enabled: bool,
    ) -> Result<()> {
        crate::instructions::set_subscription_handler(ctx, price, duration, enabled)
    }

    pub fn buy_game(ctx: Context<BuyGame>) -> Result<()> {
        crate::instructions::buy_game_handler(ctx)
    }

    pub fn withdraw(ctx: Context<Withdraw>, token: Pubkey) -> Result<()> {
        crate::instructions::withdraw_handler(ctx, token)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init, 
        payer = payer, 
        space = state::StoreConfig::SPACE, 
        seeds = [STORE_CONFIG_SEED], 
        bump
    )]
    pub store_config: Account<'info, state::StoreConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    pub pgc_game_state: Account<'info, PgcGameAccount>,
    #[account(
        init_if_needed, 
        payer = publisher, 
        space = state::PriceAccount::SPACE, 
        seeds = [PRICE_SEED, pgc_game_state.key().as_ref()], 
        bump
    )]
    pub price_account: Account<'info, state::PriceAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAffiliate<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    pub pgc_game_state: Account<'info, PgcGameAccount>,
    /// CHECK: The affiliate user
    pub affiliate: UncheckedAccount<'info>,
    #[account(
        init_if_needed, 
        payer = publisher, 
        space = state::AffiliateAccount::SPACE, 
        seeds = [AFFILIATE_SEED, pgc_game_state.key().as_ref(), affiliate.key().as_ref()], 
        bump
    )]
    pub affiliate_account: Account<'info, state::AffiliateAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetSubscription<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    pub pgc_game_state: Account<'info, PgcGameAccount>,
    #[account(
        init_if_needed, 
        payer = publisher, 
        space = state::SubscriptionAccount::SPACE, 
        seeds = [SUBSCRIPTION_SEED, pgc_game_state.key().as_ref()], 
        bump
    )]
    pub subscription_account: Account<'info, state::SubscriptionAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyGame<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(seeds = [STORE_CONFIG_SEED], bump = store_config.bump)]
    pub store_config: Account<'info, state::StoreConfig>,
    /// CHECK: Platform treasury
    #[account(mut, address = store_config.treasury)]
    pub treasury: UncheckedAccount<'info>,
    pub pgc_game_state: Account<'info, PgcGameAccount>,
    #[account(seeds = [PRICE_SEED, pgc_game_state.key().as_ref()], bump = price_account.bump)]
    pub price_account: Account<'info, state::PriceAccount>,
    
    pub affiliate_account: Option<Account<'info, state::AffiliateAccount>>,
    /// CHECK: Affiliate payout address
    #[account(mut)]
    pub affiliate: Option<UncheckedAccount<'info>>,

    #[account(
        init_if_needed, 
        payer = buyer, 
        space = state::PublisherBalanceAccount::SPACE, 
        seeds = [BALANCE_SEED, pgc_game_state.publisher.as_ref(), price_account.currency.as_ref()], 
        bump
    )]
    pub publisher_balance: Account<'info, state::PublisherBalanceAccount>,

    /// PGC1 CPI accounts
    /// CHECK: License PDA (will be created by PGC1)
    #[account(mut)]
    pub pgc_license_account: UncheckedAccount<'info>,
    pub pgc1_program: Program<'info, pgc1::program::Pgc1>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(token: Pubkey)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(mut, seeds = [STORE_CONFIG_SEED], bump = store_config.bump)]
    pub store_config: Account<'info, state::StoreConfig>,
    #[account(
        mut, 
        seeds = [BALANCE_SEED, publisher.key().as_ref(), token.as_ref()], 
        bump = publisher_balance.bump, 
        close = publisher
    )]
    pub publisher_balance: Account<'info, state::PublisherBalanceAccount>,
    pub system_program: Program<'info, System>,
}
