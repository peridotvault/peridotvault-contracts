use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::StoreInitialized,
    state::{
        StoreConfig, MAX_REFERRAL_BPS_HARD_CAP, PLATFORM_FEE_BPS_MAX,
    },
};

#[derive(Accounts)]
pub struct InitializeStore<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + StoreConfig::LEN,
        seeds = [b"store_config"],
        bump
    )]
    pub store_config: Account<'info, StoreConfig>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeStore>,
    treasury: Pubkey,
    platform_fee_bps: u16,
    default_referral_bps: u16,
    max_referral_bps: u16,
    store_actor: Pubkey,
) -> Result<()> {
    require!(platform_fee_bps <= PLATFORM_FEE_BPS_MAX, StoreError::InvalidPlatformFeeBps);
    require!(max_referral_bps <= MAX_REFERRAL_BPS_HARD_CAP, StoreError::InvalidMaxReferralBps);
    require!(default_referral_bps <= max_referral_bps, StoreError::InvalidDefaultReferralBps);
    require!((platform_fee_bps as u32 + max_referral_bps as u32) <= PLATFORM_FEE_BPS_MAX as u32, StoreError::InvalidMaxReferralBps);

    let config = &mut ctx.accounts.store_config;
    config.authority = ctx.accounts.authority.key();
    config.treasury = treasury;
    config.platform_fee_bps = platform_fee_bps;
    config.default_referral_bps = default_referral_bps;
    config.max_referral_bps = max_referral_bps;
    config.store_actor = store_actor;
    config.bump = ctx.bumps.store_config;

    emit!(StoreInitialized {
        authority: config.authority,
        treasury: config.treasury,
    });

    Ok(())
}
