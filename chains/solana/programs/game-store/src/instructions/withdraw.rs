use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{self, Mint, TokenAccount, TransferChecked},
};

use crate::{
    constants::{is_native_sol_payment_method, STORE_STATE_SEED},
    errors::GameStoreError,
    events::PublisherWithdrawn,
    states::StoreState,
};

#[derive(Accounts)]
#[instruction(token: Pubkey)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump
    )]
    pub store_state: Account<'info, StoreState>,

    pub system_program: Program<'info, System>,

    pub payment_mint: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    pub publisher_token_account: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    pub store_vault_token_account: Option<UncheckedAccount<'info>>,
    pub token_program: Option<UncheckedAccount<'info>>,
    pub associated_token_program: Option<UncheckedAccount<'info>>,
}

pub fn handler(ctx: Context<Withdraw>, token: Pubkey) -> Result<()> {
    let amount = ctx
        .accounts
        .store_state
        .take_publisher_balance(ctx.accounts.publisher.key(), token)?;

    if is_native_sol_payment_method(&token) {
        let store_state_info = ctx.accounts.store_state.to_account_info();
        let publisher_info = ctx.accounts.publisher.to_account_info();

        let rent = Rent::get()?;
        let rent_exempt_minimum = rent.minimum_balance(store_state_info.data_len());
        let current_store_lamports = **store_state_info.lamports.borrow();
        require!(
            current_store_lamports >= rent_exempt_minimum.saturating_add(amount),
            GameStoreError::InsufficientStoreLamports
        );

        **store_state_info.try_borrow_mut_lamports()? -= amount;
        **publisher_info.try_borrow_mut_lamports()? += amount;
    } else {
        let payment_mint_info = ctx
            .accounts
            .payment_mint
            .as_ref()
            .ok_or(error!(GameStoreError::InvalidPaymentMint))?;
        let publisher_token_account_info = ctx
            .accounts
            .publisher_token_account
            .as_ref()
            .ok_or(error!(GameStoreError::InvalidBuyerTokenAccount))?;
        let store_vault_token_account_info = ctx
            .accounts
            .store_vault_token_account
            .as_ref()
            .ok_or(error!(GameStoreError::InvalidStoreVaultTokenAccount))?;
        let token_program_info = ctx
            .accounts
            .token_program
            .as_ref()
            .ok_or(error!(GameStoreError::InvalidPaymentMint))?;

        let payment_mint = Mint::try_deserialize(&mut &payment_mint_info.data.borrow()[..])?;
        let store_vault_token_account = TokenAccount::try_deserialize(&mut &store_vault_token_account_info.data.borrow()[..])?;

        require_keys_eq!(
            token,
            payment_mint_info.key(),
            GameStoreError::InvalidPaymentMint
        );
        require_keys_eq!(
            store_vault_token_account.owner,
            ctx.accounts.store_state.key(),
            GameStoreError::InvalidStoreVaultTokenAccount
        );
        require_keys_eq!(
            store_vault_token_account.mint,
            payment_mint_info.key(),
            GameStoreError::InvalidPaymentMint
        );

        let store_signer_seeds: &[&[u8]] = &[STORE_STATE_SEED, &[ctx.accounts.store_state.bump]];

        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                token_program_info.to_account_info(),
                TransferChecked {
                    from: store_vault_token_account_info.to_account_info(),
                    mint: payment_mint_info.to_account_info(),
                    to: publisher_token_account_info.to_account_info(),
                    authority: ctx.accounts.store_state.to_account_info(),
                },
                &[store_signer_seeds],
            ),
            amount,
            payment_mint.decimals,
        )?;
    }

    emit!(PublisherWithdrawn {
        publisher: ctx.accounts.publisher.key(),
        token,
        amount,
    });

    Ok(())
}
