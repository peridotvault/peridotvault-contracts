use anchor_lang::prelude::*;

use crate::{
    constants::STORE_STATE_SEED,
    errors::GameStoreError,
    events::GovernanceUpdated,
    states::StoreState,
};

#[derive(Accounts)]
pub struct SetGovernance<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump,
        has_one = governance @ GameStoreError::Unauthorized
    )]
    pub store_state: Account<'info, StoreState>,
}

pub fn handler(ctx: Context<SetGovernance>, governance: Pubkey) -> Result<()> {
    require!(governance != Pubkey::default(), GameStoreError::InvalidGovernance);

    let store_state = &mut ctx.accounts.store_state;
    let old_governance = store_state.governance;
    store_state.governance = governance;

    emit!(GovernanceUpdated {
        old_governance,
        new_governance: governance,
    });

    Ok(())
}
