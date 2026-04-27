use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::PublisherUpdated,
    state::{Game, GAME_SEED},
};

#[derive(Accounts)]
pub struct SetPublisher<'info> {
    pub publisher: &'info Signer,
    #[account(mut)]
    pub game: Account<Game<'info>>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetPublisher<'info>>,
    new_publisher: Address,
) -> Result<(), ProgramError> {
    require!(new_publisher != Address::default(), PglError::Unauthorized);
    require_keys_eq!(
        ctx.accounts.game.publisher,
        *ctx.accounts.publisher.address(),
        PglError::Unauthorized
    );

    let nonce_bytes = ctx.accounts.game.nonce.get().to_le_bytes();
    quasar_lang::pda::verify_program_address(
        &[GAME_SEED, ctx.accounts.game.creator.as_ref(), &nonce_bytes],
        &crate::ID,
        ctx.accounts.game.address(),
    )?;

    let old_publisher = ctx.accounts.game.publisher;
    ctx.accounts.game.publisher = new_publisher;

    emit!(PublisherUpdated {
        game: *ctx.accounts.game.address(),
        old_publisher,
        new_publisher,
    })?;

    Ok(())
}
