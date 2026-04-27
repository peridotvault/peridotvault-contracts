use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::TreasuryUpdated,
    state::{PglConfig, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub authority: &'info Signer,
    #[account(mut, seeds = [PGL_CONFIG_SEED], bump = pgl_config.bump)]
    pub pgl_config: &'info mut Account<PglConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetTreasury<'info>>,
    treasury: Address,
) -> Result<(), ProgramError> {
    require!(treasury != Address::default(), PglError::Unauthorized);
    require_keys_eq!(
        ctx.accounts.pgl_config.authority,
        *ctx.accounts.authority.address(),
        PglError::Unauthorized
    );

    let old_treasury = ctx.accounts.pgl_config.treasury;
    ctx.accounts.pgl_config.treasury = treasury;

    emit!(TreasuryUpdated {
        authority: ctx.accounts.pgl_config.authority,
        old_treasury,
        new_treasury: treasury,
    })?;

    Ok(())
}
