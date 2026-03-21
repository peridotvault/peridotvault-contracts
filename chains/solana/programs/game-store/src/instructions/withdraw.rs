use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    constants::STORE_STATE_SEED,
    errors::GameStoreError,
    events::PublisherWithdrawn,
    states::StoreState,
};

#[derive(Accounts)]
#[instruction(_token: Pubkey)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump
    )]
    pub store_state: Account<'info, StoreState>,

    #[account(mut)]
    pub payment_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = publisher,
        associated_token::mint = payment_mint,
        associated_token::authority = publisher,
        associated_token::token_program = token_program
    )]
    pub publisher_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = store_vault_token_account.owner == store_state.key() @ GameStoreError::InvalidStoreVaultTokenAccount,
        constraint = store_vault_token_account.mint == payment_mint.key() @ GameStoreError::InvalidPaymentMint
    )]
    pub store_vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Withdraw>, token: Pubkey) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.payment_mint.key(),
        token,
        GameStoreError::InvalidPaymentMint
    );

    let amount = ctx
        .accounts
        .store_state
        .take_publisher_balance(ctx.accounts.publisher.key(), token)?;

    let store_signer_seeds: &[&[u8]] = &[STORE_STATE_SEED, &[ctx.accounts.store_state.bump]];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.store_vault_token_account.to_account_info(),
                mint: ctx.accounts.payment_mint.to_account_info(),
                to: ctx.accounts.publisher_token_account.to_account_info(),
                authority: ctx.accounts.store_state.to_account_info(),
            },
            &[store_signer_seeds],
        ),
        amount,
        ctx.accounts.payment_mint.decimals,
    )?;

    emit!(PublisherWithdrawn {
        publisher: ctx.accounts.publisher.key(),
        token,
        amount,
    });

    Ok(())
}
