use anchor_lang::prelude::*;

use crate::{events::AuthorizedRegistryProgramAdded, state::{AuthorizedRegistryProgram, StoreConfig}};

#[derive(Accounts)]
pub struct AddAuthorizedRegistryProgram<'info> {
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
        space = 8 + AuthorizedRegistryProgram::LEN,
        seeds = [b"authorized_registry_program", program_id.key().as_ref()],
        bump
    )]
    pub authorized_registry_program: Account<'info, AuthorizedRegistryProgram>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddAuthorizedRegistryProgram>) -> Result<()> {
    let account = &mut ctx.accounts.authorized_registry_program;
    account.program_id = ctx.accounts.program_id.key();
    account.active = true;
    account.bump = ctx.bumps.authorized_registry_program;

    emit!(AuthorizedRegistryProgramAdded {
        program_id: account.program_id,
    });
    Ok(())
}
