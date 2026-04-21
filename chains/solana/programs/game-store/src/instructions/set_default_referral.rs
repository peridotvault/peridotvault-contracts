use anchor_lang::prelude::*;

use crate::{errors::StoreError, events::DefaultReferralUpdated, state::StoreConfig};

#[derive(Accounts)]
pub struct SetDefaultReferral<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
}

pub fn handler(ctx: Context<SetDefaultReferral>, default_referral_bps: u16) -> Result<()> {
    let config = &mut ctx.accounts.store_config;
    require!(default_referral_bps <= config.max_referral_bps, StoreError::InvalidDefaultReferralBps);
    config.default_referral_bps = default_referral_bps;
    emit!(DefaultReferralUpdated { default_referral_bps });
    Ok(())
}
