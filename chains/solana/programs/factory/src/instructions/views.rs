use anchor_lang::prelude::*;

use crate::{constants::FACTORY_STATE_SEED, states::FactoryState};

#[derive(Accounts)]
pub struct GetFactoryView<'info> {
    #[account(
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, FactoryState>,
}

pub fn get_registry(ctx: Context<GetFactoryView>) -> Result<Pubkey> {
    Ok(ctx.accounts.factory_state.registry)
}

pub fn get_game_store(ctx: Context<GetFactoryView>) -> Result<Pubkey> {
    Ok(ctx.accounts.factory_state.game_store)
}

pub fn get_governance(ctx: Context<GetFactoryView>) -> Result<Pubkey> {
    Ok(ctx.accounts.factory_state.governance)
}
