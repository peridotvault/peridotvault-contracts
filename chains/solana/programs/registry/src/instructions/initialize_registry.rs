use anchor_lang::prelude::*;

use crate::{
    errors::RegistryError,
    events::RegistryInitialized,
    state::RegistryConfig,
};

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = RegistryConfig::SPACE,
        seeds = [b"registry_config"],
        bump
    )]
    pub config: Account<'info, RegistryConfig>,

    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<InitializeRegistry>, treasury: Pubkey) -> Result<()> {
    require!(treasury != Pubkey::default(), RegistryError::InvalidTreasury);

    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.treasury = treasury;
    config.pgl1_program = pgl1::ID;
    config.bump = ctx.bumps.config;

    emit!(RegistryInitialized {
        authority: config.authority,
        treasury: config.treasury,
        pgl1_program: config.pgl1_program,
    });

    Ok(())
}
