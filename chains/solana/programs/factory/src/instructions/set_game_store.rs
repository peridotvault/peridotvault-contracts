use anchor_lang::prelude::*;

use crate::{
    constants::FACTORY_STATE_SEED,
    errors::FactoryError,
    events::GameStoreUpdated,
    states::FactoryState,
};

#[derive(Accounts)]
pub struct SetGameStore<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
        has_one = governance @ FactoryError::Unauthorized
    )]
    pub factory_state: Account<'info, FactoryState>,
}

pub fn handler(ctx: Context<SetGameStore>, game_store: Pubkey) -> Result<()> {
    require!(game_store != Pubkey::default(), FactoryError::InvalidGameStore);

    let factory_state = &mut ctx.accounts.factory_state;
    let old_game_store = factory_state.game_store;
    factory_state.game_store = game_store;

    emit!(GameStoreUpdated {
        old_game_store,
        new_game_store: game_store,
    });

    Ok(())
}
