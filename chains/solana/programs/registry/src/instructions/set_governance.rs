use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::GovernanceUpdated,
    states::RegistryState,
};

#[derive(Accounts)]
pub struct SetGovernance<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump,
        has_one = governance @ RegistryError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
}

pub fn handler(ctx: Context<SetGovernance>, governance: Pubkey) -> Result<()> {
    require!(governance != Pubkey::default(), RegistryError::InvalidGovernance);

    let registry_state = &mut ctx.accounts.registry_state;
    let old_governance = registry_state.governance;
    registry_state.governance = governance;
    registry_state.set_admin(governance, true)?;
    if old_governance != governance {
        registry_state.set_admin(old_governance, false)?;
    }

    emit!(GovernanceUpdated {
        old_governance,
        new_governance: governance,
    });

    Ok(())
}
