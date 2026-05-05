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

    pub payment_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    /// CHECK: validated manually in handler via PDA derivation for paid path.
    pub accepted_payment_token: Option<UncheckedAccount<'info>>,
    /// CHECK: validated manually in handler via PDA derivation for paid path.
    pub game_payment_option: Option<UncheckedAccount<'info>>,
    /// CHECK: validated manually in handler for paid path.
    #[account(mut)]
    pub buyer_payment_account: Option<UncheckedAccount<'info>>,
    /// CHECK: validated manually in handler for paid path.
    #[account(mut)]
    pub publisher_payment_account: Option<UncheckedAccount<'info>>,
    /// CHECK: validated manually in handler for paid path.
    #[account(mut)]
    pub treasury_payment_account: Option<UncheckedAccount<'info>>,
    /// CHECK: validated manually in handler for paid path.
    pub referrer_payment_account: Option<UncheckedAccount<'info>>,

    /// CHECK: validated via store_config.store_actor address constraint and pgl1 authorized_actor PDA.
    #[account(
        address = store_config.store_actor @ StoreError::InvalidStoreActor
    )]
    pub store_actor: UncheckedAccount<'info>,

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

pub(crate) fn handler(
    ctx: Context<BuyGame>,
    mint_token: Option<Pubkey>,
    referrer: Option<Pubkey>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

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

    let (final_price, payment_mint_key, effective_referral_bps) =
        if let Some(chosen_mint) = mint_token {
            // ── Paid game path ──────────────────────────────────────────
            let payment_mint = ctx
                .accounts
                .payment_mint
                .as_ref()
                .ok_or(error!(StoreError::PaymentTokenNotAllowed))?;
            require_keys_eq!(
                payment_mint.key(),
                chosen_mint,
                StoreError::PaymentTokenNotAllowed
            );

            let apt_unchecked = ctx
                .accounts
                .accepted_payment_token
                .as_ref()
                .ok_or(error!(StoreError::PaymentTokenNotAllowed))?;
            let apt_data = apt_unchecked.try_borrow_data()?;
            let mut apt_slice = apt_data.as_ref();
            let accepted_payment_token = AcceptedPaymentToken::try_deserialize(&mut apt_slice)?;
            let gpo_unchecked = ctx
                .accounts
                .game_payment_option
                .as_ref()
                .ok_or(error!(StoreError::PriceNotFound))?;
            let gpo_data = gpo_unchecked.try_borrow_data()?;
            let mut gpo_slice = gpo_data.as_ref();
            let game_payment_option = GamePaymentOption::try_deserialize(&mut gpo_slice)?;

            // manual PDA checks (can't use #[account(seeds = …)] on Option<T>)
            let (expected_apt, _) = Pubkey::find_program_address(
                &[b"accepted_payment_token", payment_mint.key().as_ref()],
                ctx.program_id,
            );
            require_keys_eq!(
                apt_unchecked.key(),
                expected_apt,
                StoreError::PaymentTokenNotAllowed
            );
            require!(
                accepted_payment_token.active,
                StoreError::PaymentTokenDisabled
            );
            require_keys_eq!(
                accepted_payment_token.mint,
                payment_mint.key(),
                StoreError::PaymentTokenNotAllowed
            );

            let (expected_gpo, _) = Pubkey::find_program_address(
                &[
                    b"game_payment_option",
                    ctx.accounts.game.key().as_ref(),
                    payment_mint.key().as_ref(),
                ],
                ctx.program_id,
            );
            require_keys_eq!(
                gpo_unchecked.key(),
                expected_gpo,
                StoreError::PriceNotFound
            );
            require!(game_payment_option.active, StoreError::PriceNotFound);
            require_keys_eq!(
                game_payment_option.game,
                ctx.accounts.game.key(),
                StoreError::GamePaymentOptionMismatch
            );
            require_keys_eq!(
                game_payment_option.mint,
                payment_mint.key(),
                StoreError::PaymentTokenNotAllowed
            );

            let base_price = game_payment_option.base_price;
            let fp = compute_final_price(base_price, &ctx.accounts.game_store_config, now)?;

            // token accounts (must all be present for paid flow)
            let buyer_ata = ctx
                .accounts
                .buyer_payment_account
                .as_ref()
                .ok_or(error!(StoreError::Unauthorized))?;
            let buyer_ata_data = buyer_ata.try_borrow_data()?;
            // SPL token account layout: mint(32), owner(32), amount(8), ...
            const TOKEN_ACCOUNT_MIN_LEN: usize = 165;

            let publisher_ata = ctx
                .accounts
                .publisher_payment_account
                .as_ref()
                .ok_or(error!(StoreError::Unauthorized))?;
            let publisher_ata_data = publisher_ata.try_borrow_data()?;

            let treasury_ata = ctx
                .accounts
                .treasury_payment_account
                .as_ref()
                .ok_or(error!(StoreError::InvalidTreasury))?;
            let treasury_ata_data = treasury_ata.try_borrow_data()?;

            // Validate buyer ATA: owner=caller, mint=payment_mint
            require!(
                buyer_ata_data.len() >= TOKEN_ACCOUNT_MIN_LEN,
                StoreError::Unauthorized
            );
            let buyer_owner = Pubkey::new_from_array(
                buyer_ata_data[32..64].try_into().unwrap(),
            );
            let buyer_mint = Pubkey::new_from_array(
                buyer_ata_data[0..32].try_into().unwrap(),
            );
            require_keys_eq!(buyer_owner, ctx.accounts.buyer.key(), StoreError::Unauthorized);
            require_keys_eq!(buyer_mint, payment_mint.key(), StoreError::PaymentTokenNotAllowed);

            // Validate publisher ATA: owner=publisher, mint=payment_mint
            require!(
                publisher_ata_data.len() >= TOKEN_ACCOUNT_MIN_LEN,
                StoreError::Unauthorized
            );
            let pub_owner = Pubkey::new_from_array(
                publisher_ata_data[32..64].try_into().unwrap(),
            );
            let pub_mint = Pubkey::new_from_array(
                publisher_ata_data[0..32].try_into().unwrap(),
            );
            require_keys_eq!(pub_owner, ctx.accounts.game.publisher, StoreError::Unauthorized);
            require_keys_eq!(pub_mint, payment_mint.key(), StoreError::PaymentTokenNotAllowed);

            // Validate treasury ATA: owner=treasury, mint=payment_mint
            require!(
                treasury_ata_data.len() >= TOKEN_ACCOUNT_MIN_LEN,
                StoreError::InvalidTreasury
            );
            let treasury_owner = Pubkey::new_from_array(
                treasury_ata_data[32..64].try_into().unwrap(),
            );
            let treasury_mint = Pubkey::new_from_array(
                treasury_ata_data[0..32].try_into().unwrap(),
            );
            require_keys_eq!(treasury_owner, ctx.accounts.store_config.treasury, StoreError::InvalidTreasury);
            require_keys_eq!(treasury_mint, payment_mint.key(), StoreError::PaymentTokenNotAllowed);

            // Drop account data borrows before CPIs.
            // The Solana runtime uses RefCell-based borrow tracking. CPIs that
            // need write access to these accounts will fail if any immutable
            // borrow (from try_borrow_data) is still active.
            drop(buyer_ata_data);
            drop(publisher_ata_data);
            drop(treasury_ata_data);

            let referral = compute_effective_referral_bps(
                &ctx.accounts.store_config,
                &ctx.accounts.game_store_config,
                referrer.is_some(),
            )?;

            let platform_fee_amount =
                bps_amount(fp, ctx.accounts.store_config.platform_fee_bps)?;
            let referral_amount = if referrer.is_some() {
                bps_amount(fp, referral)?
            } else {
                0
            };
            let publisher_amount = fp
                .checked_sub(platform_fee_amount)
                .ok_or(StoreError::MathOverflow)?
                .checked_sub(referral_amount)
                .ok_or(StoreError::MathOverflow)?;

            transfer_payment(
                &ctx.accounts.token_program,
                &buyer_ata,
                payment_mint,
                &treasury_ata,
                &ctx.accounts.buyer,
                platform_fee_amount,
            )?;

            transfer_payment(
                &ctx.accounts.token_program,
                &buyer_ata,
                payment_mint,
                &publisher_ata,
                &ctx.accounts.buyer,
                publisher_amount,
            )?;

            if referral_amount > 0 {
                let referrer_key =
                    referrer.ok_or(StoreError::MissingReferrerTokenAccount)?;
                let referrer_ata = ctx
                    .accounts
                    .referrer_payment_account
                    .as_ref()
                    .ok_or(StoreError::MissingReferrerTokenAccount)?;
                let ref_data = referrer_ata.try_borrow_data()?;
                require!(
                    ref_data.len() >= TOKEN_ACCOUNT_MIN_LEN,
                    StoreError::InvalidReferrerTokenAccount
                );
                let ref_owner = Pubkey::new_from_array(
                    ref_data[32..64].try_into().unwrap(),
                );
                let ref_mint = Pubkey::new_from_array(
                    ref_data[0..32].try_into().unwrap(),
                );
                require_keys_eq!(
                    ref_owner,
                    referrer_key,
                    StoreError::InvalidReferrerTokenAccount
                );
                require_keys_eq!(
                    ref_mint,
                    payment_mint.key(),
                    StoreError::InvalidReferrerTokenAccount
                );

                transfer_payment(
                    &ctx.accounts.token_program,
                    &buyer_ata,
                    payment_mint,
                    &referrer_ata,
                    &ctx.accounts.buyer,
                    referral_amount,
                )?;
            }

            (fp, payment_mint.key(), referral)
        } else {
            // ── Free game path ──────────────────────────────────────────
            (0u64, Pubkey::default(), 0u16)
        };

    // ── License minting (common to both paths) ─────────────────────────
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

    require!(
        ctx.accounts.license.data_is_empty(),
        StoreError::AlreadyOwned
    );

    let cpi_accounts = pgl1::cpi::accounts::MintLicense {
        actor: ctx.accounts.store_actor.to_account_info(),
        holder: ctx.accounts.buyer.to_account_info(),
        authorized_actor: ctx.accounts.authorized_actor.to_account_info(),
        game: ctx.accounts.game.to_account_info(),
        license: ctx.accounts.license.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new(ctx.accounts.pgl1_program.to_account_info(), cpi_accounts);
    pgl1::cpi::mint_license(cpi_ctx, None)
        .map_err(|_| error!(StoreError::LicenseMintFailed))?;

    let referrer_key = referrer.unwrap_or(Pubkey::default());

    let receipt = &mut ctx.accounts.purchase_receipt;
    receipt.buyer = ctx.accounts.buyer.key();
    receipt.game = ctx.accounts.game.key();
    receipt.payment_mint = payment_mint_key;
    receipt.paid_amount = final_price;
    receipt.final_price = final_price;
    receipt.referrer = referrer_key;
    receipt.referral_bps_applied = effective_referral_bps;
    receipt.purchased_at = now;
    receipt.bump = ctx.bumps.purchase_receipt;

    emit!(GamePurchased {
        buyer: receipt.buyer,
        game: receipt.game,
        payment_mint: receipt.payment_mint,
        paid_amount: receipt.paid_amount,
        final_price: receipt.final_price,
        referrer: receipt.referrer,
        referral_bps_applied: receipt.referral_bps_applied,
    });

    emit!(PurchaseReceiptCreated {
        buyer: receipt.buyer,
        game: receipt.game,
        referrer: receipt.referrer,
    });

    Ok(())
}

fn transfer_payment<'info>(
    token_program: &Interface<'info, TokenInterface>,
    from: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    to: &AccountInfo<'info>,
    authority: &Signer<'info>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    let cpi_accounts = TransferChecked {
        from: from.clone(),
        mint: mint.to_account_info(),
        to: to.clone(),
        authority: authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);

    msg!("transfer_payment: from={} writable={}", from.key(), from.is_writable);
    msg!("transfer_payment: to={} writable={}", to.key(), to.is_writable);
    msg!("transfer_payment: authority={} signer={}", authority.key(), authority.is_signer);
    msg!("transfer_payment: amount={} decimals={}", amount, mint.decimals);

    transfer_checked(cpi_ctx, amount, mint.decimals)
        .map_err(|_| error!(StoreError::PaymentFailed))
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
