use anchor_lang::prelude::*;

use crate::{
    errors::RegistryError,
    events::TreasuryUpdated,
    state::RegistryConfig,
};

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,
}

pub(crate) fn handler(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
    require!(treasury != Pubkey::default(), RegistryError::InvalidTreasury);

    ctx.accounts.config.treasury = treasury;

    emit!(TreasuryUpdated { treasury });

    Ok(())
}
