use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_interface::Mint;

use crate::{
    constants::is_native_sol_payment_method,
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::RegistrationFeeUpdated,
    states::RegistryState,
};

#[derive(Accounts)]
#[instruction(_amount: u64, token: Pubkey)]
pub struct SetRegistrationFee<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump,
        has_one = governance @ RegistryError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,

    #[account(address = token)]
    pub registration_fee_mint: Option<InterfaceAccount<'info, Mint>>,
}

pub fn handler(ctx: Context<SetRegistrationFee>, amount: u64, token: Pubkey) -> Result<()> {
    require!(
        token != Pubkey::default(),
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

    let registry_state = &mut ctx.accounts.registry_state;
    let enabled = registry_state.upsert_registration_fee_option(token, amount)?;

    emit!(RegistrationFeeUpdated {
        payment_method: token,
        amount,
        enabled,
    });

    Ok(())
}
