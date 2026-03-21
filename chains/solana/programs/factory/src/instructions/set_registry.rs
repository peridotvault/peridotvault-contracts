use anchor_lang::prelude::*;

use crate::{
    constants::FACTORY_STATE_SEED,
    errors::FactoryError,
    events::RegistryUpdated,
    states::FactoryState,
};

#[derive(Accounts)]
pub struct SetRegistry<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
        has_one = governance @ FactoryError::Unauthorized
    )]
    pub factory_state: Account<'info, FactoryState>,
}

pub fn handler(ctx: Context<SetRegistry>, registry: Pubkey) -> Result<()> {
    require!(registry != Pubkey::default(), FactoryError::InvalidRegistry);

    let factory_state = &mut ctx.accounts.factory_state;
    let old_registry = factory_state.registry;
    factory_state.registry = registry;

    emit!(RegistryUpdated {
        old_registry,
        new_registry: registry,
    });

    Ok(())
}
