use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::TreasuryUpdated,
    states::RegistryState,
};

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump,
        has_one = governance @ RegistryError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
}

pub fn handler(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
    require!(treasury != Pubkey::default(), RegistryError::InvalidTreasury);

    let registry_state = &mut ctx.accounts.registry_state;
    let old_treasury = registry_state.treasury;
    registry_state.treasury = treasury;

    emit!(TreasuryUpdated {
        old_treasury,
        new_treasury: treasury,
    });

    Ok(())
}
