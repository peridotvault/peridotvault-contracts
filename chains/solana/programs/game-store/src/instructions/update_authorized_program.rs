use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::AuthorizedProgramUpdated,
    state::{AuthorizedProgram, ROLE_REGISTRY, StoreConfig},
};

#[derive(Accounts)]
pub struct UpdateAuthorizedProgram<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
    #[account(
        mut,
        seeds = [b"authorized_program", authorized_program.program_id.as_ref()],
        bump = authorized_program.bump
    )]
    pub authorized_program: Account<'info, AuthorizedProgram>,
}

pub(crate) fn handler(ctx: Context<UpdateAuthorizedProgram>, active: bool, role: Option<u8>) -> Result<()> {
    if let Some(r) = role {
        require!(r <= ROLE_REGISTRY, StoreError::InvalidRole);
        ctx.accounts.authorized_program.role = r;
    }
    ctx.accounts.authorized_program.active = active;

    emit!(AuthorizedProgramUpdated {
        program_id: ctx.accounts.authorized_program.program_id,
        active,
        role: ctx.accounts.authorized_program.role,
    });
    Ok(())
}
