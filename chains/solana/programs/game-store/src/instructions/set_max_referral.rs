use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::MaxReferralUpdated,
    state::{StoreConfig, MAX_REFERRAL_BPS_HARD_CAP, PLATFORM_FEE_BPS_MAX},
};

#[derive(Accounts)]
pub struct SetMaxReferral<'info> {
    pub authority: &'info Signer,
    #[account(
        mut,
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: &'info mut Account<StoreConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetMaxReferral<'info>>,
    max_referral_bps: u16,
) -> Result<(), ProgramError> {
    let config = &mut ctx.accounts.store_config;
    require!(
        max_referral_bps <= MAX_REFERRAL_BPS_HARD_CAP,
        StoreError::InvalidMaxReferralBps
    );
    require!(
        config.default_referral_bps.get() <= max_referral_bps,
        StoreError::InvalidMaxReferralBps
    );
    require!(
        (config.platform_fee_bps.get() as u32 + max_referral_bps as u32)
            <= PLATFORM_FEE_BPS_MAX as u32,
        StoreError::InvalidMaxReferralBps
    );
    config.max_referral_bps = max_referral_bps.into();
    emit!(MaxReferralUpdated { max_referral_bps })?;
    Ok(())
}
