use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<SetMinter>, account: Pubkey, is_authorized: bool) -> Result<()> {
    let minter_account = &mut ctx.accounts.minter_account;
    minter_account.game = ctx.accounts.game_account.key();
    minter_account.account = account;
    minter_account.is_authorized = is_authorized;
    minter_account.bump = ctx.bumps.minter_account;

    // Emitting event would be good here, but I'll add that later.
    Ok(())
}

