use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_interface::Mint;
use crate::{
    constants::is_native_sol_payment_method,
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::RegistrationFeeUpdated,
    states::{RegistrationFeeOptionAccount, RegistryState},
};

#[derive(Accounts)]
#[instruction(amount: u64, token: Pubkey)]
pub struct SetRegistrationFee<'info> {
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
        space = RegistrationFeeOptionAccount::SPACE,
        seeds = [b"fee_option", token.as_ref()],
        bump
    )]
    pub fee_option_account: Account<'info, RegistrationFeeOptionAccount>,

    pub registration_fee_mint: Option<InterfaceAccount<'info, Mint>>,

    /// CHECK: system program bypass
    #[account(address = anchor_lang::solana_program::system_program::ID)]
    pub sys_prog: Program<'info, System>,
}

pub fn handler(ctx: Context<SetRegistrationFee>, amount: u64, token: Pubkey) -> Result<()> {
    require!(
        is_native_sol_payment_method(&token) || token != Pubkey::default(),
        RegistryError::InvalidRegistrationPaymentMethod
    );
    if is_native_sol_payment_method(&token) {
        require_keys_eq!(
            token,
            system_program::ID,
            RegistryError::InvalidRegistrationPaymentMethod
        );
    } else {
        let registration_fee_mint = ctx
            .accounts
            .registration_fee_mint
            .as_ref()
            .ok_or(error!(RegistryError::MissingFeeAccounts))?;
        require_keys_eq!(
            registration_fee_mint.key(),
            token,
            RegistryError::InvalidRegistrationPaymentMethod
        );
    }

    if amount > 0 {
        let fee_option = &mut ctx.accounts.fee_option_account;
        fee_option.bump = ctx.bumps.fee_option_account;
        fee_option.payment_method = token;
        fee_option.amount = amount;
    } else {
        let account_info = ctx.accounts.fee_option_account.to_account_info();
        let dest_info = ctx.accounts.governance.to_account_info();
        
        let lamports = account_info.lamports();
        **account_info.lamports.borrow_mut() -= lamports;
        **dest_info.lamports.borrow_mut() += lamports;
        
        // Manual account closing
        account_info.assign(&System::id());
        account_info.realloc(0, false)?;
    }

    emit!(RegistrationFeeUpdated {
        payment_method: token,
        amount,
        enabled: amount > 0,
    });

    Ok(())
}
