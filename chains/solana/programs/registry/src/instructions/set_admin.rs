use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::AdminUpdated,
    states::RegistryState,
};

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump,
        has_one = governance @ RegistryError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
}

pub fn handler(ctx: Context<SetAdmin>, account: Pubkey, is_admin: bool) -> Result<()> {
    require!(account != Pubkey::default(), RegistryError::InvalidAdmin);

    let registry_state = &mut ctx.accounts.registry_state;
    registry_state.set_admin(account, is_admin)?;

    emit!(AdminUpdated { account, is_admin });

    Ok(())
}
