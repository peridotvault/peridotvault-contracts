use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::{GamePurchased, PurchaseReceiptCreated},
    state::{
        AcceptedPaymentToken, AuthorizedRegistryProgram, AuthorizedSourceProgram, BPS_DENOMINATOR,
        GamePaymentOption, GameStoreConfig, PurchaseReceipt, RegistryGameMirror,
        RegistryGameStatus, SourceGameMirror, StoreConfig,
    },
};

#[derive(Accounts)]
pub struct BuyGame<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
    )]
    pub store_config: Account<'info, StoreConfig>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_source_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedSourceProgram>,
    /// CHECK: trusted program id only
    pub source_program: UncheckedAccount<'info>,
    #[account(
        constraint = authorized_registry_program.active @ StoreError::RegistryProgramNotAuthorized,
        seeds = [b"authorized_registry_program", registry_program.key().as_ref()],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: Account<'info, AuthorizedRegistryProgram>,
    /// CHECK: trusted program id only
    pub registry_program: UncheckedAccount<'info>,
    #[account(owner = source_program.key() @ StoreError::UnsupportedSourceGameOwner)]
    pub game: Account<'info, SourceGameMirror>,
    #[account(owner = registry_program.key() @ StoreError::RegistryProgramNotAuthorized)]
    pub registry_game: Account<'info, RegistryGameMirror>,
    #[account(
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
    /// CHECK: SPL mint address only
    pub payment_mint: UncheckedAccount<'info>,
    #[account(
        constraint = accepted_payment_token.active @ StoreError::PaymentTokenDisabled,
        seeds = [b"accepted_payment_token", payment_mint.key().as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,
    #[account(
        seeds = [b"game_payment_option", game.key().as_ref(), payment_mint.key().as_ref()],
        bump = game_payment_option.bump,
        has_one = game
    )]
    pub game_payment_option: Account<'info, GamePaymentOption>,
    #[account(
        init,
        payer = buyer,
        space = 8 + PurchaseReceipt::LEN,
        seeds = [b"purchase_receipt", buyer.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub purchase_receipt: Account<'info, PurchaseReceipt>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyGame>, paid_amount: u64, _referrer: Option<Pubkey>) -> Result<()> {
    require_keys_eq!(ctx.accounts.registry_game.game, ctx.accounts.game.key(), StoreError::RegistryGameMismatch);
    require!(matches!(ctx.accounts.registry_game.status, RegistryGameStatus::Active), StoreError::GameNotActive);
    require!(ctx.accounts.game_store_config.active, StoreError::StoreGameInactive);
    require!(ctx.accounts.game_payment_option.active, StoreError::PriceNotFound);
    require_keys_eq!(ctx.accounts.accepted_payment_token.mint, ctx.accounts.payment_mint.key(), StoreError::PaymentTokenNotAllowed);
    require_keys_eq!(ctx.accounts.game_payment_option.mint, ctx.accounts.payment_mint.key(), StoreError::PaymentTokenNotAllowed);
    require!(paid_amount > 0, StoreError::InvalidPaymentAmount);

    let now = Clock::get()?.unix_timestamp;
    let base_price = ctx.accounts.game_payment_option.base_price;
    let final_price = compute_final_price(base_price, &ctx.accounts.game_store_config, now)?;
    require!(paid_amount == final_price, StoreError::InvalidPaymentAmount);

    let effective_referral_bps = compute_effective_referral_bps(&ctx.accounts.store_config, &ctx.accounts.game_store_config, _referrer.is_some())?;

    // NOTE:
    // The actual settlement transfer and CPI mint are intentionally left as integration hooks.
    // They must be wired against the exact token transfer model and exact license-program IDL.
    // This skeleton preserves the business rules and state transitions.

    let receipt = &mut ctx.accounts.purchase_receipt;
    receipt.buyer = ctx.accounts.buyer.key();
    receipt.game = ctx.accounts.game.key();
    receipt.payment_mint = ctx.accounts.payment_mint.key();
    receipt.paid_amount = paid_amount;
    receipt.final_price = final_price;
    receipt.referral_bps_applied = effective_referral_bps;
    receipt.purchased_at = now;
    receipt.bump = ctx.bumps.purchase_receipt;

    emit!(GamePurchased {
        buyer: receipt.buyer,
        game: receipt.game,
        payment_mint: receipt.payment_mint,
        paid_amount: receipt.paid_amount,
        final_price: receipt.final_price,
        referral_bps_applied: receipt.referral_bps_applied,
    });

    emit!(PurchaseReceiptCreated {
        buyer: receipt.buyer,
        game: receipt.game,
    });

    Ok(())
}

fn compute_final_price(base_price: u64, cfg: &GameStoreConfig, now: i64) -> Result<u64> {
    let discount_bps = match cfg.discount_bps {
        None => return Ok(base_price),
        Some(bps) => bps,
    };
    require!(discount_bps <= 10_000, StoreError::InvalidDiscountBps);

    if let Some(start) = cfg.discount_starts_at {
        if now < start {
            return Ok(base_price);
        }
    }
    if let Some(end) = cfg.discount_expires_at {
        if now > end {
            return Ok(base_price);
        }
    }

    let discount_amount = (base_price as u128)
        .checked_mul(discount_bps as u128)
        .ok_or(StoreError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(StoreError::MathOverflow)? as u64;

    base_price.checked_sub(discount_amount).ok_or(StoreError::MathOverflow.into())
}

fn compute_effective_referral_bps(
    store_config: &StoreConfig,
    cfg: &GameStoreConfig,
    has_referrer: bool,
) -> Result<u16> {
    if !has_referrer {
        return Ok(0);
    }

    let effective = match cfg.referral_bps {
        None => store_config.default_referral_bps,
        Some(0) => store_config.default_referral_bps,
        Some(v) => v,
    };

    require!(effective <= store_config.max_referral_bps, StoreError::ReferralAboveMax);
    require!((store_config.platform_fee_bps as u32 + effective as u32) <= 10_000, StoreError::InvalidReferralBps);
    Ok(effective)
}
