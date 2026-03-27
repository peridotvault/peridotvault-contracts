use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::AdminUpdated,
    states::{AdminAccount, RegistryState},
};

#[derive(Accounts)]
#[instruction(account: Pubkey, is_admin: bool)]
pub struct SetAdmin<'info> {
    #[account(mut)]
    pub governance: Signer<'info>,

    #[account(
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump,
        has_one = governance @ RegistryError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,

    #[account(
        init_if_needed,
        payer = governance,
        space = AdminAccount::SPACE,
        seeds = [b"admin", account.as_ref()],
        bump
    )]
    pub admin_account: Account<'info, AdminAccount>,

    /// CHECK: system program bypass
    #[account(address = anchor_lang::solana_program::system_program::ID)]
    pub sys_prog: Program<'info, System>,
}

pub fn handler(ctx: Context<SetAdmin>, account: Pubkey, is_admin: bool) -> Result<()> {
    require!(account != Pubkey::default(), RegistryError::InvalidAdmin);

    if is_admin {
        let admin_account = &mut ctx.accounts.admin_account;
        admin_account.bump = ctx.bumps.admin_account;
        admin_account.admin = account;
    } else {
        // If we want to remove, we close the account.
        // Anchor's #[account(close)] is static at the derive level.
        // To do it dynamically, we can use the close_account helper or separate instructions.
        // For simplicity and since Registry is small, I'll split into add/remove if needed,
        // but for now I'll use manual closing in the handler to keep the interface same as before.
        let account_info = ctx.accounts.admin_account.to_account_info();
        let dest_info = ctx.accounts.governance.to_account_info();
        
        // Manual account closing
        let lamports = account_info.lamports();
        **account_info.lamports.borrow_mut() -= lamports;
        **dest_info.lamports.borrow_mut() += lamports;
        account_info.assign(&System::id());
        account_info.realloc(0, false)?;
    }

    emit!(AdminUpdated { account, is_admin });

    Ok(())
}
