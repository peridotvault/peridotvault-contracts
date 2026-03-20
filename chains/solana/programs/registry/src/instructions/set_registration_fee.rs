use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
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
    pub registration_fee_mint: InterfaceAccount<'info, Mint>,
}

pub fn handler(ctx: Context<SetRegistrationFee>, amount: u64, token: Pubkey) -> Result<()> {
    require!(
        token != Pubkey::default(),
        RegistryError::InvalidRegistrationFeeToken
    );
    require_keys_eq!(
        ctx.accounts.registration_fee_mint.key(),
        token,
        RegistryError::InvalidRegistrationFeeToken
    );

    let registry_state = &mut ctx.accounts.registry_state;
    registry_state.registration_fee = amount;
    registry_state.registration_fee_token = token;

    emit!(RegistrationFeeUpdated { amount, token });

    Ok(())
}
