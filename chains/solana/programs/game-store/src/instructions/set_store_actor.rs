use anchor_lang::prelude::*;

use crate::{errors::StoreError, events::StoreActorUpdated, state::StoreConfig};

#[derive(Accounts)]
pub struct SetStoreActor<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority @ StoreError::Unauthorized
    )]
    pub store_config: Account<'info, StoreConfig>,
}

pub(crate) fn handler(ctx: Context<SetStoreActor>, new_store_actor: Pubkey) -> Result<()> {
    require!(new_store_actor != Pubkey::default(), StoreError::InvalidStoreActor);

    let config = &mut ctx.accounts.store_config;
    let old_store_actor = config.store_actor;
    config.store_actor = new_store_actor;

    emit!(StoreActorUpdated {
        old_store_actor,
        new_store_actor,
    });

    Ok(())
}
