use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GamePaymentOptionRemoved,
    state::{AuthorizedSourceProgram, GamePaymentOption},
};

#[derive(Accounts)]
pub struct RemoveGamePaymentOption<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_source_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedSourceProgram>,
    pub source_program: Program<'info, pgl1::program::Pgl1>,
    pub game: Account<'info, pgl1::state::Game>,
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
    emit!(GamePaymentOptionRemoved {
        game: ctx.accounts.game.key(),
        mint: ctx.accounts.mint.key(),
    });
    Ok(())
}
