use anchor_lang::prelude::*;
use crate::SetSubscription;

pub fn handler(
    ctx: Context<SetSubscription>,
    price: u64,
    duration: i64,
    enabled: bool,
) -> Result<()> {
    let sub_account = &mut ctx.accounts.subscription_account;
    sub_account.game = ctx.accounts.pgc_game_state.key();
    sub_account.price = price;
    sub_account.duration = duration;
    sub_account.enabled = enabled;
    sub_account.bump = ctx.bumps.subscription_account;

    Ok(())
}
