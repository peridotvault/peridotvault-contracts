use anchor_lang::prelude::*;

use crate::{
    constants::FACTORY_STATE_SEED,
    errors::FactoryError,
    events::FactoryInitialized,
    states::FactoryState,
};

#[derive(Accounts)]
#[instruction(governance: Pubkey, registry: Pubkey, game_store: Pubkey)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = FactoryState::SPACE,
        seeds = [FACTORY_STATE_SEED],
        bump
    )]
    pub factory_state: Account<'info, FactoryState>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    governance: Pubkey,
    registry: Pubkey,
    game_store: Pubkey,
) -> Result<()> {
    require!(governance != Pubkey::default(), FactoryError::InvalidGovernance);
    require!(registry != Pubkey::default(), FactoryError::InvalidRegistry);
    require!(game_store != Pubkey::default(), FactoryError::InvalidGameStore);

    let factory_state = &mut ctx.accounts.factory_state;
    factory_state.bump = ctx.bumps.factory_state;
    factory_state.registry = registry;
    factory_state.game_store = game_store;
    factory_state.governance = governance;

    emit!(FactoryInitialized {
        governance,
        registry,
        game_store,
    });

    Ok(())
}
