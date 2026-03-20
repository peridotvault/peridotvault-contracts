use anchor_lang::prelude::*;

use crate::{
    constants::*,
    errors::Pgc1Error,
    events::MinterUpdated,
    states::{GameState, MinterAuthority},
};

#[derive(Accounts)]
pub struct SetMinter<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        has_one = publisher @ Pgc1Error::Unauthorized
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: target minter
    pub account: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = publisher,
        space = MinterAuthority::SPACE,
        seeds = [MINTER_AUTH_SEED, game_state.key().as_ref(), account.key().as_ref()],
        bump
    )]
    pub minter_auth: Account<'info, MinterAuthority>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SetMinter>, is_authorized: bool) -> Result<()> {
    let account = ctx.accounts.account.key();
    require!(account != Pubkey::default(), Pgc1Error::InvalidMinter);

    let minter_auth = &mut ctx.accounts.minter_auth;
    minter_auth.bump = ctx.bumps.minter_auth;
    minter_auth.game = ctx.accounts.game_state.key();
    minter_auth.account = account;
    minter_auth.is_authorized = is_authorized;

    emit!(MinterUpdated {
        game: ctx.accounts.game_state.key(),
        account,
        is_authorized,
    });

    Ok(())
}
