use anchor_lang::prelude::*;

use crate::{errors::Pgc1Error, events::PublisherUpdated, states::GameState};

#[derive(Accounts)]
pub struct SetPublisher<'info> {
    pub publisher: Signer<'info>,

    #[account(
        mut,
        has_one = publisher @ Pgc1Error::Unauthorized
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: new canonical publisher
    pub new_publisher: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<SetPublisher>) -> Result<()> {
    let new_publisher = ctx.accounts.new_publisher.key();
    require!(
        new_publisher != Pubkey::default(),
        Pgc1Error::InvalidPublisher
    );

    let game_state = &mut ctx.accounts.game_state;
    let old_publisher = game_state.publisher;
    game_state.publisher = new_publisher;

    emit!(PublisherUpdated {
        game: game_state.key(),
        old_publisher,
        new_publisher,
    });

    Ok(())
}
