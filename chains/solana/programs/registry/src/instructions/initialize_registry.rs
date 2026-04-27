use crate::{
    errors::RegistryError,
    events::RegistryInitialized,
    external::PGL1_ID,
    state::{RegistryConfig, REGISTRY_CONFIG_SEED},
};
use quasar_lang::prelude::*;
#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    pub authority: &'info mut Signer,
    #[account(init, payer=authority, space=<RegistryConfig as Space>::SPACE, seeds=[REGISTRY_CONFIG_SEED], bump)]
    pub config: &'info mut Account<RegistryConfig>,
    pub system_program: &'info Program<System>,
}
pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, InitializeRegistry<'info>>,
    treasury: Address,
) -> Result<(), ProgramError> {
    require!(
        treasury != Address::default(),
        RegistryError::InvalidTreasury
    );
    ctx.accounts.config.set_inner(
        *ctx.accounts.authority.address(),
        treasury,
        PGL1_ID,
        ctx.bumps.config,
    );
    emit!(RegistryInitialized {
        authority: *ctx.accounts.authority.address(),
        treasury,
        pgl1_program: PGL1_ID
    })?;
    Ok(())
}
