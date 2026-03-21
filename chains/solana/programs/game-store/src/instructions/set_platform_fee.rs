use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_FEE_BPS, STORE_STATE_SEED},
    errors::GameStoreError,
    events::PlatformFeeUpdated,
    states::StoreState,
};

#[derive(Accounts)]
pub struct SetPlatformFee<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump,
        has_one = governance @ GameStoreError::Unauthorized
    )]
    pub store_state: Account<'info, StoreState>,
}

pub fn handler(ctx: Context<SetPlatformFee>, fee_bps: u16) -> Result<()> {
    require!(fee_bps <= MAX_FEE_BPS, GameStoreError::InvalidPlatformFeeBps);

    ctx.accounts.store_state.platform_fee_bps = fee_bps;

    emit!(PlatformFeeUpdated {
        platform_fee_bps: fee_bps,
    });

    Ok(())
}
