use anchor_lang::{prelude::*, system_program};

use crate::{
    constants::STORE_STATE_SEED,
    errors::GameStoreError,
    events::NativeSolPublisherWithdrawn,
    states::StoreState,
};

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump
    )]
    pub store_state: Account<'info, StoreState>,
}

pub fn handler(ctx: Context<WithdrawSol>) -> Result<()> {
    let amount = ctx
        .accounts
        .store_state
        .take_publisher_balance(ctx.accounts.publisher.key(), system_program::ID)?;

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

    emit!(NativeSolPublisherWithdrawn {
        publisher: ctx.accounts.publisher.key(),
        amount,
    });

    Ok(())
}
