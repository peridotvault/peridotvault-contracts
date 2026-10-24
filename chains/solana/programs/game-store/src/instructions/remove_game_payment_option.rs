use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GamePaymentOptionRemoved,
    state::{AuthorizedProgram, GamePaymentOption, GameStoreConfig},
};

#[derive(Accounts)]
pub struct RemoveGamePaymentOption<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedProgram>,
    pub source_program: Program<'info, pgl1::program::Pgl1>,
    pub game: Account<'info, pgl1::state::Game>,
    /// H3 guard counter: closing an ACTIVE option decrements the count so a
    /// later `set_game_free_flag(true)` observes the true state.
    #[account(
        mut,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
    /// CHECK: mint PDA seed input
    pub mint: UncheckedAccount<'info>,
    #[account(
        mut,
        close = publisher,
        seeds = [b"game_payment_option", game.key().as_ref(), mint.key().as_ref()],
        bump = game_payment_option.bump,
        has_one = game
    )]
    pub game_payment_option: Account<'info, GamePaymentOption>,
}

pub(crate) fn handler(ctx: Context<RemoveGamePaymentOption>) -> Result<()> {
    require_keys_eq!(ctx.accounts.game.publisher, ctx.accounts.publisher.key(), StoreError::Unauthorized);
    require_keys_eq!(ctx.accounts.game_payment_option.mint, ctx.accounts.mint.key(), StoreError::PaymentTokenNotAllowed);
    // H3 guard counter: an ACTIVE option disappearing must decrement the
    // count (checked: underflow means corrupted state, fail rather than wrap).
    // Runs before the `close` constraint transfers lamports; the account data
    // is still readable here.
    if ctx.accounts.game_payment_option.active {
        ctx.accounts.game_store_config.active_payment_options = ctx
            .accounts
            .game_store_config
            .active_payment_options
            .checked_sub(1)
            .ok_or(StoreError::MathOverflow)?;
    }
    emit!(GamePaymentOptionRemoved {
        game: ctx.accounts.game.key(),
        mint: ctx.accounts.mint.key(),
    });
    Ok(())
}
