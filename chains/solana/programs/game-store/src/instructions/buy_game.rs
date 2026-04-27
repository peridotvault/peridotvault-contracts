use quasar_lang::{prelude::*, sysvars::Sysvar};
use quasar_spl::{InterfaceAccount, Mint, Token, TokenCpi, TokenInterface};

use crate::{
    errors::StoreError,
    events::{GamePurchased, PurchaseReceiptCreated},
    external::{
        self, assert_active_registry_game, Pgl1Program, PglAuthorizedActor, PglGame, RegistryGame,
        RegistryProgram, AUTHORIZED_ACTOR_SEED, LICENSE_SEED,
    },
    instructions::{read_option_address, read_u64},
    state::{
        AcceptedPaymentToken, AuthorizedProgram, GamePaymentOption, GameStoreConfig,
        PurchaseReceipt, StoreConfig, BPS_DENOMINATOR,
    },
};

#[derive(Accounts)]
pub struct BuyGame<'info> {
    pub buyer: &'info mut Signer,

    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
    )]
    pub store_config: &'info Account<StoreConfig>,

    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,
    pub source_program: &'info Program<Pgl1Program>,

    #[account(
        constraint = authorized_registry_program.active.get() @ StoreError::RegistryProgramNotAuthorized,
        seeds = [b"authorized_program", registry_program],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: &'info Account<AuthorizedProgram>,
    pub registry_program: &'info Program<RegistryProgram>,

    pub game: &'info Account<PglGame>,
    pub registry_game: &'info Account<RegistryGame>,

    #[account(
        seeds = [b"game_store_config", game],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: &'info Account<GameStoreConfig>,

    pub payment_mint: &'info InterfaceAccount<Mint>,

    #[account(
        constraint = accepted_payment_token.active.get() @ StoreError::PaymentTokenDisabled,
        seeds = [b"accepted_payment_token", payment_mint],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: &'info Account<AcceptedPaymentToken>,

    #[account(
        seeds = [b"game_payment_option", game, payment_mint],
        bump = game_payment_option.bump,
        has_one = game
    )]
    pub game_payment_option: &'info Account<GamePaymentOption>,

    #[account(
        mut,
        constraint = *buyer_payment_account.owner() == *buyer.address() @ StoreError::Unauthorized,
        constraint = *buyer_payment_account.mint() == *payment_mint.address() @ StoreError::PaymentTokenNotAllowed
    )]
    pub buyer_payment_account: &'info mut InterfaceAccount<Token>,

    #[account(
        mut,
        constraint = *publisher_payment_account.owner() == game.publisher()? @ StoreError::Unauthorized,
        constraint = *publisher_payment_account.mint() == *payment_mint.address() @ StoreError::PaymentTokenNotAllowed
    )]
    pub publisher_payment_account: &'info mut InterfaceAccount<Token>,

    #[account(
        mut,
        constraint = *treasury_payment_account.owner() == store_config.treasury @ StoreError::InvalidTreasury,
        constraint = *treasury_payment_account.mint() == *payment_mint.address() @ StoreError::PaymentTokenNotAllowed
    )]
    pub treasury_payment_account: &'info mut InterfaceAccount<Token>,

    pub referrer_payment_account: Option<&'info mut InterfaceAccount<Token>>,

    #[account(address = store_config.store_actor @ StoreError::InvalidStoreActor)]
    pub store_actor: &'info Signer,

    pub authorized_actor: &'info Account<PglAuthorizedActor>,

    /// CHECK: duplicate of `source_program`; read-only program account used for legacy account ordering.
    #[account(dup)]
    pub pgl1_program: &'info Program<Pgl1Program>,

    pub license: &'info mut UncheckedAccount,

    #[account(
        init_if_needed,
        payer = buyer,
        space = <PurchaseReceipt as Space>::SPACE,
        seeds = [b"purchase_receipt", buyer, game],
        bump,
    )]
    pub purchase_receipt: &'info mut Account<PurchaseReceipt>,

    pub token_program: &'info Interface<TokenInterface>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(ctx: &mut Ctx<'info, BuyGame<'info>>) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let paid_amount = read_u64(ctx.data, &mut offset)?;
    let referrer = read_option_address(ctx.data, &mut offset)?;

    require_keys_eq!(
        ctx.accounts.registry_game.game()?,
        *ctx.accounts.game.address(),
        StoreError::RegistryGameMismatch
    );
    assert_active_registry_game(ctx.accounts.registry_game)?;
    require!(
        ctx.accounts.game_store_config.active.get(),
        StoreError::StoreGameInactive
    );
    require!(
        ctx.accounts.game_payment_option.active.get(),
        StoreError::PriceNotFound
    );
    require_keys_eq!(
        ctx.accounts.accepted_payment_token.mint,
        *ctx.accounts.payment_mint.address(),
        StoreError::PaymentTokenNotAllowed
    );
    require_keys_eq!(
        ctx.accounts.game_payment_option.mint,
        *ctx.accounts.payment_mint.address(),
        StoreError::PaymentTokenNotAllowed
    );
    require!(paid_amount > 0, StoreError::InvalidPaymentAmount);

    let actor_bump = ctx.accounts.authorized_actor.bump()?;
    let actor_bump_ref = [actor_bump];
    quasar_lang::pda::verify_program_address(
        &[
            AUTHORIZED_ACTOR_SEED,
            ctx.accounts.store_actor.address().as_ref(),
            &actor_bump_ref,
        ],
        ctx.accounts.pgl1_program.address(),
        ctx.accounts.authorized_actor.address(),
    )?;
    require_keys_eq!(
        ctx.accounts.authorized_actor.actor()?,
        *ctx.accounts.store_actor.address(),
        StoreError::Unauthorized
    );
    require!(
        ctx.accounts.authorized_actor.active()?,
        StoreError::Unauthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let base_price = ctx.accounts.game_payment_option.base_price.get();
    let final_price = compute_final_price(base_price, ctx.accounts.game_store_config, now.get())?;
    require!(paid_amount == final_price, StoreError::InvalidPaymentAmount);

    let referrer_key = referrer.unwrap_or_default();

    let effective_referral_bps = compute_effective_referral_bps(
        ctx.accounts.store_config,
        ctx.accounts.game_store_config,
        referrer.is_some(),
    )?;

    let platform_fee_amount = bps_amount(
        final_price,
        ctx.accounts.store_config.platform_fee_bps.get(),
    )?;
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
        ctx.accounts.token_program,
        ctx.accounts.buyer_payment_account,
        ctx.accounts.payment_mint,
        ctx.accounts.treasury_payment_account,
        ctx.accounts.buyer,
        platform_fee_amount,
    )?;

    transfer_payment(
        ctx.accounts.token_program,
        ctx.accounts.buyer_payment_account,
        ctx.accounts.payment_mint,
        ctx.accounts.publisher_payment_account,
        ctx.accounts.buyer,
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
            *referrer_payment_account.owner(),
            referrer_key,
            StoreError::InvalidReferrerTokenAccount
        );
        require_keys_eq!(
            *referrer_payment_account.mint(),
            *ctx.accounts.payment_mint.address(),
            StoreError::InvalidReferrerTokenAccount
        );

        transfer_payment(
            ctx.accounts.token_program,
            ctx.accounts.buyer_payment_account,
            ctx.accounts.payment_mint,
            *referrer_payment_account,
            ctx.accounts.buyer,
            referral_amount,
        )?;
    }

    let (expected_license, _) = quasar_lang::pda::based_try_find_program_address(
        &[
            LICENSE_SEED,
            ctx.accounts.buyer.address().as_ref(),
            ctx.accounts.game.address().as_ref(),
        ],
        ctx.accounts.pgl1_program.address(),
    )?;
    require_keys_eq!(
        *ctx.accounts.license.address(),
        expected_license,
        StoreError::LicenseMintFailed
    );
    require!(
        ctx.accounts.license.to_account_view().data_len() == 0,
        StoreError::AlreadyOwned
    );

    external::mint_license(
        ctx.accounts.pgl1_program,
        ctx.accounts.store_actor,
        ctx.accounts.buyer,
        ctx.accounts.authorized_actor,
        ctx.accounts.game,
        ctx.accounts.license,
        ctx.accounts.system_program,
    )
    .map_err(|_| StoreError::LicenseMintFailed)?;

    ctx.accounts.purchase_receipt.set_inner(
        *ctx.accounts.buyer.address(),
        *ctx.accounts.game.address(),
        *ctx.accounts.payment_mint.address(),
        paid_amount,
        final_price,
        referrer_key,
        effective_referral_bps,
        now.get(),
        ctx.bumps.purchase_receipt,
    );

    emit!(GamePurchased {
        buyer: *ctx.accounts.buyer.address(),
        game: *ctx.accounts.game.address(),
        payment_mint: *ctx.accounts.payment_mint.address(),
        paid_amount,
        final_price,
        referrer: referrer_key,
        referral_bps_applied: effective_referral_bps,
    })?;

    emit!(PurchaseReceiptCreated {
        buyer: *ctx.accounts.buyer.address(),
        game: *ctx.accounts.game.address(),
        referrer: referrer_key,
    })?;

    Ok(())
}

fn transfer_payment(
    token_program: &Interface<TokenInterface>,
    from: &InterfaceAccount<Token>,
    mint: &InterfaceAccount<Mint>,
    to: &InterfaceAccount<Token>,
    authority: &Signer,
    amount: u64,
) -> Result<(), ProgramError> {
    if amount == 0 {
        return Ok(());
    }

    token_program
        .transfer_checked(from, mint, to, authority, amount, mint.decimals())
        .invoke()
        .map_err(|_| StoreError::PaymentFailed.into())
}

fn bps_amount(amount: u64, bps: u16) -> Result<u64, ProgramError> {
    ((amount as u128)
        .checked_mul(bps as u128)
        .ok_or(StoreError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(StoreError::MathOverflow)?)
    .try_into()
    .map_err(|_| StoreError::MathOverflow.into())
}

fn compute_final_price(
    base_price: u64,
    cfg: &GameStoreConfig,
    now: i64,
) -> Result<u64, ProgramError> {
    let discount_bps = match cfg.discount_bps.get() {
        None => return Ok(base_price),
        Some(bps) => bps,
    };
    require!(discount_bps <= 10_000, StoreError::InvalidDiscountBps);

    if let Some(start) = cfg.discount_starts_at.get() {
        if now < start {
            return Ok(base_price);
        }
    }
    if let Some(end) = cfg.discount_expires_at.get() {
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
) -> Result<u16, ProgramError> {
    if !has_referrer {
        return Ok(0);
    }

    let effective = match cfg.referral_bps.get() {
        None => store_config.default_referral_bps.get(),
        Some(0) => store_config.default_referral_bps.get(),
        Some(v) => v,
    };

    require!(
        effective <= store_config.max_referral_bps.get(),
        StoreError::ReferralAboveMax
    );
    require!(
        (store_config.platform_fee_bps.get() as u32 + effective as u32) <= 10_000,
        StoreError::InvalidReferralBps
    );
    Ok(effective)
}
