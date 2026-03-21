use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::{
    constants::is_native_sol_payment_method,
    errors::RegistryError,
    states::RegistryState,
};

pub mod initialize;
pub mod register_game;
pub mod register_game_by_factory;
pub mod set_admin;
pub mod set_factory;
pub mod set_fee_exemption;
pub mod set_governance;
pub mod set_registration_fee;
pub mod set_status;
pub mod set_treasury;

pub(crate) fn collect_registration_fee<'info>(
    registry_state: &RegistryState,
    publisher: Pubkey,
    payment_method: Pubkey,
    authority: AccountInfo<'info>,
    treasury: AccountInfo<'info>,
    payer_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
    treasury_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
    fee_payment_mint: Option<&InterfaceAccount<'info, Mint>>,
    token_program: Option<&Interface<'info, TokenInterface>>,
    system_program_account: AccountInfo<'info>,
) -> Result<()> {
    if registry_state.registration_fee_options.is_empty() || registry_state.is_fee_exempt(&publisher)
    {
        return Ok(());
    }

    let fee_option = registry_state
        .registration_fee_option(&payment_method)
        .ok_or(error!(RegistryError::RegistrationFeeOptionNotFound))?;

    require_keys_eq!(
        treasury.key(),
        registry_state.treasury,
        RegistryError::InvalidTreasuryAccount
    );

    if is_native_sol_payment_method(&payment_method) {
        system_program::transfer(
            CpiContext::new(
                system_program_account,
                Transfer {
                    from: authority,
                    to: treasury,
                },
            ),
            fee_option.amount,
        )?;
        return Ok(());
    }

    let payer_token_account =
        payer_token_account.ok_or(error!(RegistryError::MissingFeeAccounts))?;
    let treasury_token_account =
        treasury_token_account.ok_or(error!(RegistryError::MissingFeeAccounts))?;
    let fee_payment_mint = fee_payment_mint.ok_or(error!(RegistryError::MissingFeeAccounts))?;
    let token_program = token_program.ok_or(error!(RegistryError::MissingFeeAccounts))?;

    require_keys_eq!(
        fee_payment_mint.key(),
        payment_method,
        RegistryError::InvalidRegistrationPaymentMethod
    );
    require_keys_eq!(
        payer_token_account.mint,
        fee_payment_mint.key(),
        RegistryError::RegistrationFeeMintMismatch
    );
    require_keys_eq!(
        treasury_token_account.mint,
        fee_payment_mint.key(),
        RegistryError::RegistrationFeeMintMismatch
    );
    require_keys_eq!(
        payer_token_account.owner,
        authority.key(),
        RegistryError::InvalidFeePayerTokenAccount
    );
    require_keys_eq!(
        treasury_token_account.owner,
        registry_state.treasury,
        RegistryError::InvalidTreasuryTokenAccount
    );

    token_interface::transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                from: payer_token_account.to_account_info(),
                mint: fee_payment_mint.to_account_info(),
                to: treasury_token_account.to_account_info(),
                authority,
            },
        ),
        fee_option.amount,
        fee_payment_mint.decimals,
    )
}
