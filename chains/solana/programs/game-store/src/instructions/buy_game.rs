use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{
    errors::StoreError,
    events::{GamePurchased, PurchaseReceiptCreated},
    state::{
        AcceptedPaymentToken, AuthorizedProgram, BPS_DENOMINATOR,
        GamePaymentOption, GameStoreConfig, PurchaseReceipt, StoreConfig,
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
    pub store_config: Box<Account<'info, StoreConfig>>,

    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Box<Account<'info, AuthorizedProgram>>,
    pub source_program: Program<'info, pgl1::program::Pgl1>,

    #[account(
        constraint = authorized_registry_program.active @ StoreError::RegistryProgramNotAuthorized,
        seeds = [b"authorized_program", registry_program.key().as_ref()],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: Box<Account<'info, AuthorizedProgram>>,
    pub registry_program: Program<'info, registry_program::program::Registry>,

    pub game: Box<Account<'info, pgl1::state::Game>>,
    pub registry_game: Box<Account<'info, registry_program::state::RegistryGame>>,

    #[account(
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Box<Account<'info, GameStoreConfig>>,

    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        constraint = accepted_payment_token.active @ StoreError::PaymentTokenDisabled,
        seeds = [b"accepted_payment_token", payment_mint.key().as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Box<Account<'info, AcceptedPaymentToken>>,

    #[account(
        seeds = [b"game_payment_option", game.key().as_ref(), payment_mint.key().as_ref()],
        bump = game_payment_option.bump,
        has_one = game
    )]
    pub game_payment_option: Box<Account<'info, GamePaymentOption>>,

    #[account(
        mut,
        constraint = buyer_payment_account.owner == buyer.key() @ StoreError::Unauthorized,
        constraint = buyer_payment_account.mint == payment_mint.key() @ StoreError::PaymentTokenNotAllowed
    )]
    pub buyer_payment_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = publisher_payment_account.owner == game.publisher @ StoreError::Unauthorized,
        constraint = publisher_payment_account.mint == payment_mint.key() @ StoreError::PaymentTokenNotAllowed
    )]
    pub publisher_payment_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = treasury_payment_account.owner == store_config.treasury @ StoreError::InvalidTreasury,
        constraint = treasury_payment_account.mint == payment_mint.key() @ StoreError::PaymentTokenNotAllowed
    )]
    pub treasury_payment_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub referrer_payment_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,

    #[account(
        address = store_config.store_actor @ StoreError::InvalidStoreActor
    )]
    pub store_actor: Signer<'info>,

    #[account(
        seeds = [pgl1::state::AUTHORIZED_ACTOR_SEED, store_actor.key().as_ref()],
        seeds::program = pgl1_program.key(),
        bump = authorized_actor.bump,
        constraint = authorized_actor.actor == store_actor.key() @ StoreError::Unauthorized,
        constraint = authorized_actor.active @ StoreError::Unauthorized
    )]
    pub authorized_actor: Box<Account<'info, pgl1::state::AuthorizedActor>>,

    pub pgl1_program: Program<'info, pgl1::program::Pgl1>,

    /// CHECK: validated against derived license PDA and ownership checks in handler.
    #[account(mut)]
    pub license: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = PurchaseReceipt::SPACE,
        seeds = [b"purchase_receipt", buyer.key().as_ref(), game.key().as_ref()],
        bump,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<BuyGame>, paid_amount: u64, referrer: Option<Pubkey>) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.registry_game.game,
        ctx.accounts.game.key(),
        StoreError::RegistryGameMismatch
    );
    require!(
        matches!(
            ctx.accounts.registry_game.status,
            registry_program::state::GameStatus::Active
        ),
        StoreError::GameNotActive
    );
    require!(ctx.accounts.game_store_config.active, StoreError::StoreGameInactive);
    require!(ctx.accounts.game_payment_option.active, StoreError::PriceNotFound);
    require_keys_eq!(
        ctx.accounts.accepted_payment_token.mint,
        ctx.accounts.payment_mint.key(),
        StoreError::PaymentTokenNotAllowed
    );
    require_keys_eq!(
        ctx.accounts.game_payment_option.mint,
        ctx.accounts.payment_mint.key(),
        StoreError::PaymentTokenNotAllowed
    );
    require!(paid_amount > 0, StoreError::InvalidPaymentAmount);

    let now = Clock::get()?.unix_timestamp;
    let base_price = ctx.accounts.game_payment_option.base_price;
    let final_price = compute_final_price(base_price, &ctx.accounts.game_store_config, now)?;
    require!(paid_amount == final_price, StoreError::InvalidPaymentAmount);

    let effective_referral_bps = compute_effective_referral_bps(
        &ctx.accounts.store_config,
        &ctx.accounts.game_store_config,
        referrer.is_some(),
    )?;

    let platform_fee_amount = bps_amount(final_price, ctx.accounts.store_config.platform_fee_bps)?;
    let referral_amount = if referrer.is_some() {
        bps_amount(final_price, effective_referral_bps)?
    } else {
        0
    };
    let publisher_amount = final_price
        .checked_sub(platform_fee_amount)
        .ok_or(StoreError::MathOverflow)?
        .checked_sub(referral_amount)
        .ok_or(StoreError::MathOverflow)?;

    transfer_payment(
        &ctx.accounts.token_program,
        &ctx.accounts.buyer_payment_account,
        &ctx.accounts.payment_mint,
        &ctx.accounts.treasury_payment_account,
        &ctx.accounts.buyer,
        platform_fee_amount,
    )?;

    transfer_payment(
        &ctx.accounts.token_program,
        &ctx.accounts.buyer_payment_account,
        &ctx.accounts.payment_mint,
        &ctx.accounts.publisher_payment_account,
        &ctx.accounts.buyer,
        publisher_amount,
    )?;

    if referral_amount > 0 {
        let referrer_key = referrer.ok_or(StoreError::MissingReferrerTokenAccount)?;
        let referrer_payment_account = ctx
            .accounts
            .referrer_payment_account
            .as_ref()
            .ok_or(StoreError::MissingReferrerTokenAccount)?;

        require_keys_eq!(
            referrer_payment_account.owner,
            referrer_key,
            StoreError::InvalidReferrerTokenAccount
        );
        require_keys_eq!(
            referrer_payment_account.mint,
            ctx.accounts.payment_mint.key(),
            StoreError::InvalidReferrerTokenAccount
        );

        transfer_payment(
            &ctx.accounts.token_program,
            &ctx.accounts.buyer_payment_account,
            &ctx.accounts.payment_mint,
            referrer_payment_account,
            &ctx.accounts.buyer,
            referral_amount,
        )?;
    }

    let (expected_license, _) = Pubkey::find_program_address(
        &[
            pgl1::state::LICENSE_SEED,
            ctx.accounts.buyer.key().as_ref(),
            ctx.accounts.game.key().as_ref(),
        ],
        &ctx.accounts.pgl1_program.key(),
    );
    require_keys_eq!(
        ctx.accounts.license.key(),
        expected_license,
        StoreError::LicenseMintFailed
    );

    require!(ctx.accounts.license.data_is_empty(), StoreError::AlreadyOwned);

    let cpi_accounts = pgl1::cpi::accounts::MintLicense {
        actor: ctx.accounts.store_actor.to_account_info(),
        holder: ctx.accounts.buyer.to_account_info(),
        authorized_actor: ctx.accounts.authorized_actor.to_account_info(),
        game: ctx.accounts.game.to_account_info(),
        license: ctx.accounts.license.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.pgl1_program.to_account_info(), cpi_accounts);
    pgl1::cpi::mint_license(cpi_ctx, None).map_err(|_| error!(StoreError::LicenseMintFailed))?;

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

fn transfer_payment<'info>(
    token_program: &Interface<'info, TokenInterface>,
    from: &InterfaceAccount<'info, TokenAccount>,
    mint: &InterfaceAccount<'info, Mint>,
    to: &InterfaceAccount<'info, TokenAccount>,
    authority: &Signer<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let cpi_accounts = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);

    transfer_checked(cpi_ctx, amount, mint.decimals).map_err(|_| error!(StoreError::PaymentFailed))
}

fn bps_amount(amount: u64, bps: u16) -> Result<u64> {
    ((amount as u128)
        .checked_mul(bps as u128)
        .ok_or(StoreError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(StoreError::MathOverflow)?)
    .try_into()
    .map_err(|_| error!(StoreError::MathOverflow))
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

    base_price
        .checked_sub(discount_amount)
        .ok_or(StoreError::MathOverflow.into())
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

    require!(
        effective <= store_config.max_referral_bps,
        StoreError::ReferralAboveMax
    );
    require!(
        (store_config.platform_fee_bps as u32 + effective as u32) <= 10_000,
        StoreError::InvalidReferralBps
    );
    Ok(effective)
}
