use anchor_lang::prelude::*;

use crate::{events::AuthorizedSourceProgramUpdated, state::{AuthorizedSourceProgram, StoreConfig}};

#[derive(Accounts)]
pub struct UpdateAuthorizedSourceProgram<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
    #[account(
        mut,
        seeds = [b"authorized_source_program", authorized_source_program.program_id.as_ref()],
        bump = authorized_source_program.bump
    )]
    pub authorized_source_program: Account<'info, AuthorizedSourceProgram>,
}

pub(crate) fn handler(ctx: Context<UpdateAuthorizedSourceProgram>, active: bool) -> Result<()> {
    let account = &mut ctx.accounts.authorized_source_program;
    account.active = active;

    emit!(AuthorizedSourceProgramUpdated {
        program_id: account.program_id,
        active,
    });
    Ok(())
}
