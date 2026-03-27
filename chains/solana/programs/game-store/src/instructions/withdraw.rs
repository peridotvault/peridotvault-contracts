use anchor_lang::prelude::*;
use crate::events::Withdrawal;
use crate::Withdraw;

pub fn handler(ctx: Context<Withdraw>, _token: Pubkey) -> Result<()> {
    let amount = ctx.accounts.publisher_balance.amount;
    require!(amount > 0, crate::errors::GameStoreError::EmptyPublisherBalance);

    // SOL withdrawal from PDA vault
    let config_info = ctx.accounts.store_config.to_account_info();
    let publisher_info = ctx.accounts.publisher.to_account_info();
    
    **config_info.try_borrow_mut_lamports()? -= amount;
    **publisher_info.try_borrow_mut_lamports()? += amount;

    ctx.accounts.publisher_balance.amount = 0;

    emit!(Withdrawal {
        publisher: ctx.accounts.publisher.key(),
        token: _token,
        amount,
    });

    Ok(())
}
