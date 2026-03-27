use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::FeeExemptionUpdated,
    states::{FeeExemptionAccount, RegistryState},
};

#[derive(Accounts)]
#[instruction(account: Pubkey, is_exempt: bool)]
pub struct SetFeeExemption<'info> {
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
        space = FeeExemptionAccount::SPACE,
        seeds = [b"fee_exemption", account.as_ref()],
        bump
    )]
    pub fee_exemption_account: Account<'info, FeeExemptionAccount>,

    /// CHECK: system program bypass
    #[account(address = anchor_lang::solana_program::system_program::ID)]
    pub sys_prog: Program<'info, System>,
}

pub fn handler(ctx: Context<SetFeeExemption>, account: Pubkey, is_exempt: bool) -> Result<()> {
    require!(
        account != Pubkey::default(),
        RegistryError::InvalidFeeExemptionAccount
    );

    if is_exempt {
        let fee_exemption = &mut ctx.accounts.fee_exemption_account;
        fee_exemption.bump = ctx.bumps.fee_exemption_account;
        fee_exemption.account = account;
    } else {
        let account_info = ctx.accounts.fee_exemption_account.to_account_info();
        let dest_info = ctx.accounts.governance.to_account_info();
        
        let current_lamports = account_info.lamports();
        **account_info.lamports.borrow_mut() -= current_lamports;
        **dest_info.lamports.borrow_mut() += current_lamports;
        account_info.assign(&System::id());
        account_info.realloc(0, false)?;
    }

    emit!(FeeExemptionUpdated { account, is_exempt });

    Ok(())
}
