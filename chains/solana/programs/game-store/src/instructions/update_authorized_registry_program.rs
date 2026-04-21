use anchor_lang::prelude::*;

use crate::{events::AuthorizedRegistryProgramUpdated, state::{AuthorizedRegistryProgram, StoreConfig}};

#[derive(Accounts)]
pub struct UpdateAuthorizedRegistryProgram<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
    #[account(
        mut,
        seeds = [b"authorized_registry_program", authorized_registry_program.program_id.as_ref()],
        bump = authorized_registry_program.bump
    )]
    pub authorized_registry_program: Account<'info, AuthorizedRegistryProgram>,
}

pub fn handler(ctx: Context<UpdateAuthorizedRegistryProgram>, active: bool) -> Result<()> {
    let account = &mut ctx.accounts.authorized_registry_program;
    account.active = active;

    emit!(AuthorizedRegistryProgramUpdated {
        program_id: account.program_id,
        active,
    });
    Ok(())
}
