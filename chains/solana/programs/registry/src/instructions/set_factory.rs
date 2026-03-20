use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::FactoryUpdated,
    states::RegistryState,
};

#[derive(Accounts)]
pub struct SetFactory<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump,
        has_one = governance @ RegistryError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
}

pub fn handler(ctx: Context<SetFactory>, factory: Pubkey) -> Result<()> {
    require!(factory != Pubkey::default(), RegistryError::InvalidFactory);

    let registry_state = &mut ctx.accounts.registry_state;
    let old_factory = registry_state.factory;
    registry_state.factory = factory;

    emit!(FactoryUpdated {
        old_factory,
        new_factory: factory,
    });

    Ok(())
}
