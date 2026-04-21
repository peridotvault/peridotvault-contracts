use anchor_lang::prelude::*;

use crate::{
    errors::PglError,
    events::PglInitialized,
    state::{PglConfig, PGL_CONFIG_SEED},
};

pub fn handler(
    ctx: Context<InitializePgl>,
    treasury: Pubkey,
    create_game_fee_lamports: u64,
) -> Result<()> {
    require!(treasury != Pubkey::default(), PglError::Unauthorized);

    let config = &mut ctx.accounts.pgl_config;
    config.authority = ctx.accounts.authority.key();
    config.treasury = treasury;
    config.create_game_fee_lamports = create_game_fee_lamports;
    config.bump = ctx.bumps.pgl_config;

    emit!(PglInitialized {
        authority: config.authority,
        treasury: config.treasury,
        create_game_fee_lamports: config.create_game_fee_lamports,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePgl<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = PglConfig::SPACE,
        seeds = [PGL_CONFIG_SEED],
        bump,
    )]
    pub pgl_config: Account<'info, PglConfig>,

    pub system_program: Program<'info, System>,
}
