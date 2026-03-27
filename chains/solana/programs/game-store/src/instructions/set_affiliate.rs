use anchor_lang::prelude::*;
use crate::constants::*;
use crate::SetAffiliate;

pub fn handler(
    ctx: Context<SetAffiliate>,
    share_bps: u16,
) -> Result<()> {
    require!(share_bps <= MAX_BPS, crate::errors::GameStoreError::InvalidDiscountBps);
    
    let affiliate_account = &mut ctx.accounts.affiliate_account;
    affiliate_account.game = ctx.accounts.pgc_game_state.key();
    affiliate_account.affiliate = ctx.accounts.affiliate.key();
    affiliate_account.share_bps = share_bps;
    affiliate_account.bump = ctx.bumps.affiliate_account;

    Ok(())
}
