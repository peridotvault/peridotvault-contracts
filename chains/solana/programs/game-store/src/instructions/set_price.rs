use anchor_lang::prelude::*;
use crate::SetPrice;

pub fn handler(
    ctx: Context<SetPrice>,
    price: u64,
    currency: Pubkey,
) -> Result<()> {
    let price_account = &mut ctx.accounts.price_account;
    price_account.game = ctx.accounts.pgc_game_state.key();
    price_account.price = price;
    price_account.currency = currency;
    price_account.bump = ctx.bumps.price_account;

    Ok(())
}
