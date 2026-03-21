use anchor_lang::prelude::*;

use crate::{
    constants::{is_native_sol_payment_method, REGISTRY_STATE_SEED},
    errors::RegistryError,
    events::{RegistrationFeeOptionEvent, RegistryInitialized},
    states::{RegistrationFeeOption, RegistryState},
};

#[derive(Accounts)]
#[instruction(governance: Pubkey, treasury: Pubkey, factory: Pubkey)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = RegistryState::SPACE,
        seeds = [REGISTRY_STATE_SEED],
        bump
    )]
    pub registry_state: Account<'info, RegistryState>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    governance: Pubkey,
    treasury: Pubkey,
    factory: Pubkey,
    registration_fee: u64,
    registration_fee_token: Pubkey,
) -> Result<()> {
    require!(governance != Pubkey::default(), RegistryError::InvalidGovernance);
    require!(treasury != Pubkey::default(), RegistryError::InvalidTreasury);
    require!(factory != Pubkey::default(), RegistryError::InvalidFactory);
    if registration_fee > 0 {
        require!(
            is_native_sol_payment_method(&registration_fee_token)
                || registration_fee_token != Pubkey::default(),
            RegistryError::InvalidRegistrationPaymentMethod
        );
    }

    let registry_state = &mut ctx.accounts.registry_state;
    registry_state.bump = ctx.bumps.registry_state;
    registry_state.governance = governance;
    registry_state.treasury = treasury;
    registry_state.factory = factory;
    registry_state.registration_fee_options = if registration_fee > 0 {
        vec![RegistrationFeeOption {
            payment_method: registration_fee_token,
            amount: registration_fee,
        }]
    } else {
        Vec::new()
    };
    registry_state.admins = vec![governance];
    registry_state.fee_exemptions = Vec::new();
    registry_state.games = Vec::new();
    registry_state.all_game_ids = Vec::new();

    emit!(RegistryInitialized {
        governance,
        treasury,
        factory,
        registration_fee_options: registry_state
            .registration_fee_options
            .iter()
            .map(|entry| RegistrationFeeOptionEvent {
                payment_method: entry.payment_method,
                amount: entry.amount,
            })
            .collect(),
    });

    Ok(())
}
