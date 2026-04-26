use anchor_lang::prelude::*;

use crate::{events::AuthorizedSourceProgramAdded, state::{AuthorizedSourceProgram, StoreConfig}};

#[derive(Accounts)]
pub struct AddAuthorizedSourceProgram<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
    /// CHECK: program id to authorize
    pub program_id: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = AuthorizedSourceProgram::SPACE,
        seeds = [b"authorized_source_program", program_id.key().as_ref()],
        bump
    )]
    pub authorized_source_program: Account<'info, AuthorizedSourceProgram>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<AddAuthorizedSourceProgram>) -> Result<()> {
    let account = &mut ctx.accounts.authorized_source_program;
    account.program_id = ctx.accounts.program_id.key();
    account.active = true;
    account.bump = ctx.bumps.authorized_source_program;

    emit!(AuthorizedSourceProgramAdded {
        program_id: account.program_id,
    });
    Ok(())
}
