use crate::{
    errors::RegistryError,
    events::TreasuryUpdated,
    state::{RegistryConfig, REGISTRY_CONFIG_SEED},
};
use quasar_lang::prelude::*;
#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub authority: &'info Signer,
    #[account(mut, seeds=[REGISTRY_CONFIG_SEED], bump=config.bump, has_one=authority)]
    pub config: &'info mut Account<RegistryConfig>,
}
pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetTreasury<'info>>,
    treasury: Address,
) -> Result<(), ProgramError> {
    require!(
        treasury != Address::default(),
        RegistryError::InvalidTreasury
    );
    let old_treasury = ctx.accounts.config.treasury;
    ctx.accounts.config.treasury = treasury;
    emit!(TreasuryUpdated {
        old_treasury,
        new_treasury: treasury
    })?;
    Ok(())
}
