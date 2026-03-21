use anchor_lang::prelude::*;

use crate::{
    constants::FACTORY_STATE_SEED,
    errors::FactoryError,
    events::GovernanceUpdated,
    states::FactoryState,
};

#[derive(Accounts)]
pub struct SetGovernance<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
        has_one = governance @ FactoryError::Unauthorized
    )]
    pub factory_state: Account<'info, FactoryState>,
}

pub fn handler(ctx: Context<SetGovernance>, governance: Pubkey) -> Result<()> {
    require!(governance != Pubkey::default(), FactoryError::InvalidGovernance);

    let factory_state = &mut ctx.accounts.factory_state;
    let old_governance = factory_state.governance;
    factory_state.governance = governance;

    emit!(GovernanceUpdated {
        old_governance,
        new_governance: governance,
    });

    Ok(())
}
