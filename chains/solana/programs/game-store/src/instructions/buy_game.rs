use anchor_lang::prelude::*;
use crate::BuyGame;
use pgc1::cpi::accounts::MintLicense;

pub fn handler(ctx: Context<BuyGame>) -> Result<()> {
    let price_account = &ctx.accounts.price_account;
    let final_price = price_account.final_price();
    let config = &ctx.accounts.store_config;

    // Fees calculation
    let platform_fee = (u128::from(final_price) * u128::from(config.platform_fee_bps)) / 10_000;
    
    let mut affiliate_share = 0u128;
    if let Some(affiliate_account) = &ctx.accounts.affiliate_account {
        affiliate_share = (u128::from(final_price) * u128::from(affiliate_account.share_bps)) / 10_000;
    }

    let publisher_revenue = final_price.saturating_sub((platform_fee + affiliate_share) as u64);

    // Transfer platform fee
    anchor_lang::solana_program::program::invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &config.treasury,
            platform_fee as u64,
        ),
        &[ctx.accounts.buyer.to_account_info(), ctx.accounts.treasury.to_account_info()],
    )?;

    // Transfer affiliate share
    if affiliate_share > 0 {
        if let Some(affiliate_info) = &ctx.accounts.affiliate {
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.buyer.key(),
                    &affiliate_info.key(),
                    affiliate_share as u64,
                ),
                &[ctx.accounts.buyer.to_account_info(), affiliate_info.to_account_info()],
            )?;
        }
    }

    // Record publisher balance (VAULT Patter)
    let balance = &mut ctx.accounts.publisher_balance;
    balance.amount = balance.amount.saturating_add(publisher_revenue);

    // Transfer to vault (PDA)
    anchor_lang::solana_program::program::invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &config.key(),
            publisher_revenue,
        ),
        &[ctx.accounts.buyer.to_account_info(), config.to_account_info()],
    )?;

    // CPI to PGC1
    let cpi_program = ctx.accounts.pgc1_program.to_account_info();
    let cpi_accounts = MintLicense {
        minter: ctx.accounts.store_config.to_account_info(),
        minter_account: ctx.accounts.pgc_minter_account.to_account_info(),
        game: ctx.accounts.pgc_game_state.to_account_info(),
        user: ctx.accounts.buyer.to_account_info(),
        license_account: ctx.accounts.pgc_license_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let seeds = &[
        crate::constants::STORE_CONFIG_SEED,
        &[ctx.accounts.store_config.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    // 1 year duration
    let expires_at = Clock::get()?.unix_timestamp + 365 * 24 * 60 * 60;
    pgc1::cpi::mint_license(cpi_ctx, expires_at)?;

    Ok(())
}
