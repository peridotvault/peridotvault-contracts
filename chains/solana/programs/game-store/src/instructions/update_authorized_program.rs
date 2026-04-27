use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::AuthorizedProgramUpdated,
    instructions::{read_bool, read_option_u8},
    state::{AuthorizedProgram, StoreConfig, ROLE_REGISTRY},
};

#[derive(Accounts)]
pub struct UpdateAuthorizedProgram<'info> {
    pub authority: &'info Signer,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: &'info Account<StoreConfig>,
    #[account(mut)]
    pub authorized_program: &'info mut Account<AuthorizedProgram>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, UpdateAuthorizedProgram<'info>>,
) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let active = read_bool(ctx.data, &mut offset)?;
    let role = read_option_u8(ctx.data, &mut offset)?;

    if let Some(r) = role {
        require!(r <= ROLE_REGISTRY, StoreError::InvalidRole);
        ctx.accounts.authorized_program.role = r;
    }
    ctx.accounts.authorized_program.active = active.into();

    emit!(AuthorizedProgramUpdated {
        program_id: ctx.accounts.authorized_program.program_id,
        active,
        role: ctx.accounts.authorized_program.role,
    })?;
    Ok(())
}
