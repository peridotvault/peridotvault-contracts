use quasar_lang::prelude::*;

use crate::{
    errors::PglError,
    events::AuthorityUpdated,
    state::{PglConfig, PGL_CONFIG_SEED},
};

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    pub authority: &'info Signer,
    #[account(
        mut,
        seeds = [PGL_CONFIG_SEED],
        bump = pgl_config.bump,
        has_one = authority
    )]
    pub pgl_config: &'info mut Account<PglConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetAuthority<'info>>,
    new_authority: Address,
) -> Result<(), ProgramError> {
    require!(new_authority != Address::default(), PglError::Unauthorized);

    let old_authority = ctx.accounts.pgl_config.authority;
    ctx.accounts.pgl_config.authority = new_authority;

    emit!(AuthorityUpdated {
        old_authority,
        new_authority,
    })?;

    Ok(())
}
