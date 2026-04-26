use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::AuthorizedProgramAdded,
    state::{AuthorizedProgram, ROLE_REGISTRY, StoreConfig},
};

#[derive(Accounts)]
pub struct AddAuthorizedProgram<'info> {
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
        space = AuthorizedProgram::SPACE,
        seeds = [b"authorized_program", program_id.key().as_ref()],
        bump
    )]
    pub authorized_program: Account<'info, AuthorizedProgram>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<AddAuthorizedProgram>, role: u8) -> Result<()> {
    require!(role <= ROLE_REGISTRY, StoreError::InvalidRole);

    let account = &mut ctx.accounts.authorized_program;
    account.program_id = ctx.accounts.program_id.key();
    account.active = true;
    account.role = role;
    account.bump = ctx.bumps.authorized_program;

    emit!(AuthorizedProgramAdded {
        program_id: account.program_id,
        role,
    });
    Ok(())
}
