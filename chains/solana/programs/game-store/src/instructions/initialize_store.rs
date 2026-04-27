use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::StoreInitialized,
    state::{StoreConfig, MAX_REFERRAL_BPS_HARD_CAP, PLATFORM_FEE_BPS_MAX},
};

#[derive(Accounts)]
pub struct InitializeStore<'info> {
    pub authority: &'info mut Signer,
    #[account(
        init,
        payer = authority,
        space = <StoreConfig as Space>::SPACE,
        seeds = [b"store_config"],
        bump
    )]
    pub store_config: &'info mut Account<StoreConfig>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, InitializeStore<'info>>,
    treasury: Address,
    platform_fee_bps: u16,
    default_referral_bps: u16,
    max_referral_bps: u16,
    store_actor: Address,
) -> Result<(), ProgramError> {
    require!(treasury != Address::default(), StoreError::InvalidTreasury);
    require!(
        store_actor != Address::default(),
        StoreError::InvalidStoreActor
    );
    require!(
        platform_fee_bps <= PLATFORM_FEE_BPS_MAX,
        StoreError::InvalidPlatformFeeBps
    );
    require!(
        max_referral_bps <= MAX_REFERRAL_BPS_HARD_CAP,
        StoreError::InvalidMaxReferralBps
    );
    require!(
        default_referral_bps <= max_referral_bps,
        StoreError::InvalidDefaultReferralBps
    );
    require!(
        (platform_fee_bps as u32 + max_referral_bps as u32) <= PLATFORM_FEE_BPS_MAX as u32,
        StoreError::InvalidMaxReferralBps
    );

    ctx.accounts.store_config.set_inner(
        *ctx.accounts.authority.address(),
        treasury,
        platform_fee_bps,
        default_referral_bps,
        max_referral_bps,
        store_actor,
        ctx.bumps.store_config,
    );

    emit!(StoreInitialized {
        authority: *ctx.accounts.authority.address(),
        treasury,
    })?;

    Ok(())
}
