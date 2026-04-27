use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::PlatformFeeUpdated,
    state::{StoreConfig, PLATFORM_FEE_BPS_MAX},
};

#[derive(Accounts)]
pub struct SetPlatformFee<'info> {
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
    ctx: &mut Ctx<'info, SetPlatformFee<'info>>,
    platform_fee_bps: u16,
) -> Result<(), ProgramError> {
    require!(
        platform_fee_bps <= PLATFORM_FEE_BPS_MAX,
        StoreError::InvalidPlatformFeeBps
    );
    require!(
        (platform_fee_bps as u32 + ctx.accounts.store_config.max_referral_bps.get() as u32)
            <= PLATFORM_FEE_BPS_MAX as u32,
        StoreError::InvalidPlatformFeeBps
    );

    ctx.accounts.store_config.platform_fee_bps = platform_fee_bps.into();
    emit!(PlatformFeeUpdated { platform_fee_bps })?;
    Ok(())
}
