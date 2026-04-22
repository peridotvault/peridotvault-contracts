use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::PublisherUpdated,
    state::{Game, GAME_SEED},
};

pub(crate) fn handler(ctx: Context<SetPublisher>, new_publisher: Pubkey) -> Result<()> {
    require!(new_publisher != Pubkey::default(), PglError::Unauthorized);

    let game = &mut ctx.accounts.game;
    let signer = ctx.accounts.publisher.key();
    require_keys_eq!(game.publisher, signer, PglError::Unauthorized);

    let old_publisher = game.publisher;
    game.publisher = new_publisher;

    emit!(PublisherUpdated {
        game: game.key(),
        old_publisher,
        new_publisher,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetPublisher<'info> {
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_SEED, game.creator.as_ref(), &game.nonce.to_le_bytes()],
        bump = game.bump,
    )]
    pub game: Account<'info, Game>,
}
