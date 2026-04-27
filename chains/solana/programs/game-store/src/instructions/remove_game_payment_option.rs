use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GamePaymentOptionRemoved,
    external::{Pgl1Program, PglGame},
    state::{AuthorizedProgram, GamePaymentOption},
};

#[derive(Accounts)]
pub struct RemoveGamePaymentOption<'info> {
    pub publisher: &'info mut Signer,
    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,
    pub source_program: &'info Program<Pgl1Program>,
    pub game: &'info Account<PglGame>,
    pub mint: &'info UncheckedAccount,
    #[account(
        mut,
        close = publisher,
        seeds = [b"game_payment_option", game, mint],
        bump = game_payment_option.bump,
        has_one = game
    )]
    pub game_payment_option: &'info mut Account<GamePaymentOption>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, RemoveGamePaymentOption<'info>>,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.game.publisher()?,
        *ctx.accounts.publisher.address(),
        StoreError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.game_payment_option.mint,
        *ctx.accounts.mint.address(),
        StoreError::PaymentTokenNotAllowed
    );
    emit!(GamePaymentOptionRemoved {
        game: *ctx.accounts.game.address(),
        mint: *ctx.accounts.mint.address(),
    })?;
    Ok(())
}
