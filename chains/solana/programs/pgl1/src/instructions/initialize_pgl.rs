use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::PglInitialized,
    state::{PglConfig, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct InitializePgl<'info> {
    pub authority: &'info mut Signer,
    #[account(
        init,
        payer = authority,
        space = <PglConfig as Space>::SPACE,
        seeds = [PGL_CONFIG_SEED],
        bump
    )]
    pub pgl_config: &'info mut Account<PglConfig>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, InitializePgl<'info>>,
    treasury: Address,
    create_game_fee_lamports: u64,
) -> Result<(), ProgramError> {
    require!(treasury != Address::default(), PglError::Unauthorized);

    ctx.accounts.pgl_config.set_inner(
        *ctx.accounts.authority.address(),
        treasury,
        create_game_fee_lamports,
        ctx.bumps.pgl_config,
    );

    emit!(PglInitialized {
        authority: *ctx.accounts.authority.address(),
        treasury,
        create_game_fee_lamports,
    })?;

    Ok(())
}
