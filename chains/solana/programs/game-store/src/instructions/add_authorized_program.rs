use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::AuthorizedProgramAdded,
    state::{AuthorizedProgram, StoreConfig, ROLE_REGISTRY},
};

#[derive(Accounts)]
pub struct AddAuthorizedProgram<'info> {
    pub authority: &'info mut Signer,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: &'info Account<StoreConfig>,
    pub program_id: &'info UncheckedAccount,
    #[account(
        init,
        payer = authority,
        space = <AuthorizedProgram as Space>::SPACE,
        seeds = [b"authorized_program", program_id],
        bump
    )]
    pub authorized_program: &'info mut Account<AuthorizedProgram>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, AddAuthorizedProgram<'info>>,
    role: u8,
) -> Result<(), ProgramError> {
    require!(role <= ROLE_REGISTRY, StoreError::InvalidRole);

    ctx.accounts.authorized_program.set_inner(
        *ctx.accounts.program_id.address(),
        true,
        role,
        ctx.bumps.authorized_program,
    );

    emit!(AuthorizedProgramAdded {
        program_id: *ctx.accounts.program_id.address(),
        role,
    })?;
    Ok(())
}
