use anchor_lang::prelude::*;

use crate::{errors::StoreError, events::PlatformFeeUpdated, state::{StoreConfig, PLATFORM_FEE_BPS_MAX}};

#[derive(Accounts)]
pub struct SetPlatformFee<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
}

pub fn handler(ctx: Context<SetPlatformFee>, platform_fee_bps: u16) -> Result<()> {
    require!(platform_fee_bps <= PLATFORM_FEE_BPS_MAX, StoreError::InvalidPlatformFeeBps);
    require!((platform_fee_bps as u32 + ctx.accounts.store_config.max_referral_bps as u32) <= PLATFORM_FEE_BPS_MAX as u32, StoreError::InvalidPlatformFeeBps);

    ctx.accounts.store_config.platform_fee_bps = platform_fee_bps;
    emit!(PlatformFeeUpdated { platform_fee_bps });
    Ok(())
}
