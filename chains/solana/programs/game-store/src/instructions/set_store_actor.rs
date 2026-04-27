use quasar_lang::prelude::*;

use crate::{errors::StoreError, events::StoreActorUpdated, state::StoreConfig};

#[derive(Accounts)]
pub struct SetStoreActor<'info> {
    pub authority: &'info Signer,

    #[account(
        mut,
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority @ StoreError::Unauthorized
    )]
    pub store_config: &'info mut Account<StoreConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetStoreActor<'info>>,
    new_store_actor: Address,
) -> Result<(), ProgramError> {
    require!(
        new_store_actor != Address::default(),
        StoreError::InvalidStoreActor
    );

    let config = &mut ctx.accounts.store_config;
    let old_store_actor = config.store_actor;
    config.store_actor = new_store_actor;

    emit!(StoreActorUpdated {
        old_store_actor,
        new_store_actor,
    })?;

    Ok(())
}
