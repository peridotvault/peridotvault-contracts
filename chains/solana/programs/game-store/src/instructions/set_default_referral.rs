use quasar_lang::prelude::*;

use crate::{errors::StoreError, events::DefaultReferralUpdated, state::StoreConfig};

#[derive(Accounts)]
pub struct SetDefaultReferral<'info> {
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
    ctx: &mut Ctx<'info, SetDefaultReferral<'info>>,
    default_referral_bps: u16,
) -> Result<(), ProgramError> {
    let config = &mut ctx.accounts.store_config;
    require!(
        default_referral_bps <= config.max_referral_bps.get(),
        StoreError::InvalidDefaultReferralBps
    );
    config.default_referral_bps = default_referral_bps.into();
    emit!(DefaultReferralUpdated {
        default_referral_bps
    })?;
    Ok(())
}
