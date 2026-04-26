use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::TreasuryUpdated,
    state::{PglConfig, PGL_CONFIG_SEED},
};

pub(crate) fn handler(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
    require!(treasury != Pubkey::default(), PglError::Unauthorized);

    let config = &mut ctx.accounts.pgl_config;
    require_keys_eq!(config.authority, ctx.accounts.authority.key(), PglError::Unauthorized);

    let old_treasury = config.treasury;
    config.treasury = treasury;

    emit!(TreasuryUpdated {
        authority: config.authority,
        old_treasury,
        new_treasury: treasury,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [PGL_CONFIG_SEED],
        bump = pgl_config.bump,
    )]
    pub pgl_config: Account<'info, PglConfig>,
}
