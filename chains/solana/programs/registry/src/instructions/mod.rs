use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::{errors::RegistryError, states::RegistryState};

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
pub mod views;

pub(crate) fn collect_registration_fee<'info>(
    registry_state: &RegistryState,
    publisher: Pubkey,
    authority: AccountInfo<'info>,
    payer_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
    treasury_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
    registration_fee_mint: Option<&InterfaceAccount<'info, Mint>>,
    token_program: Option<&Interface<'info, TokenInterface>>,
) -> Result<()> {
    if registry_state.registration_fee == 0 || registry_state.is_fee_exempt(&publisher) {
        return Ok(());
    }

    let payer_token_account =
        payer_token_account.ok_or(error!(RegistryError::MissingFeeAccounts))?;
    let treasury_token_account =
        treasury_token_account.ok_or(error!(RegistryError::MissingFeeAccounts))?;
    let registration_fee_mint =
        registration_fee_mint.ok_or(error!(RegistryError::MissingFeeAccounts))?;
    let token_program = token_program.ok_or(error!(RegistryError::MissingFeeAccounts))?;

    require_keys_eq!(
        registration_fee_mint.key(),
        registry_state.registration_fee_token,
        RegistryError::InvalidRegistrationFeeToken
    );
    require_keys_eq!(
        payer_token_account.mint,
        registration_fee_mint.key(),
        RegistryError::RegistrationFeeMintMismatch
    );
    require_keys_eq!(
        treasury_token_account.mint,
        registration_fee_mint.key(),
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
                mint: registration_fee_mint.to_account_info(),
                to: treasury_token_account.to_account_info(),
                authority,
            },
        ),
        registry_state.registration_fee,
        registration_fee_mint.decimals,
    )
}
