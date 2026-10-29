use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GamePaymentOptionSet,
    state::{
        AcceptedPaymentToken, AuthorizedProgram, GamePaymentOption,
        GameStoreConfig, CURRENT_VERSION, ROLE_REGISTRY,
    },
};

#[derive(Accounts)]
pub struct SetGamePaymentOption<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// H-3 (Access control, least privilege): game publisher must sign.
    /// The permissionless fallback (registry-role-only, `publisher=None`)
    /// is removed: price changes without the publisher's signature are
    /// rejected by construction (fail-closed).
    pub publisher: Signer<'info>,

    pub source_program: Program<'info, pgl1::program::Pgl1>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        constraint = authorized_source_program.role == 0 @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedProgram>,

    pub registry_program: Program<'info, registry_program::program::Registry>,
    #[account(
        constraint = authorized_registry_program.active @ StoreError::RegistryProgramNotAuthorized,
        constraint = authorized_registry_program.role >= ROLE_REGISTRY @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", registry_program.key().as_ref()],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: Account<'info, AuthorizedProgram>,

    pub game: Account<'info, pgl1::state::Game>,
    /// CHECK: registry-owned account; validated via PDA derivation
    #[account(
        owner = registry_program.key() @ StoreError::RegistryProgramNotAuthorized,
    )]
    pub registry_game: UncheckedAccount<'info>,
    // H3 guard counter lives here: mutated on every option transition.
    #[account(
        mut,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
    /// CHECK: SPL mint address only
    pub mint: UncheckedAccount<'info>,
    #[account(
        constraint = accepted_payment_token.active @ StoreError::PaymentTokenDisabled,
        seeds = [b"accepted_payment_token", mint.key().as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,
    #[account(
        init_if_needed,
        payer = payer,
        space = GamePaymentOption::SPACE,
        seeds = [b"game_payment_option", game.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub game_payment_option: Account<'info, GamePaymentOption>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<SetGamePaymentOption>, base_price: u64, active: bool) -> Result<()> {
    require!(base_price > 0, StoreError::InvalidPrice);
    // M5 version gate.
    require!(
        ctx.accounts.game_store_config.version <= CURRENT_VERSION,
        StoreError::UnsupportedVersion
    );
    // H3 fail-closed: a free game cannot gain an ACTIVE payment option.
    // Deactivation/cleanup on a free game is allowed; authority must clear
    // the flag before re-adding paid options.
    if active {
        require!(
            !ctx.accounts.game_store_config.is_free,
            StoreError::FreeGameHasPaymentOptions
        );
    }

    let publisher_key = ctx.accounts.game.publisher;

    // H-3: publisher signature required — no permissionless fallback.
    require_keys_eq!(
        ctx.accounts.publisher.key(),
        publisher_key,
        StoreError::Unauthorized
    );

    require_keys_eq!(ctx.accounts.registry_game.key(), {
        let (pda, _) = Pubkey::find_program_address(
            &[b"registry_game", ctx.accounts.game.key().as_ref()],
            &ctx.accounts.registry_program.key(),
        );
        pda
    }, StoreError::RegistryGameMismatch);
    require!(ctx.accounts.game_store_config.active, StoreError::StoreGameInactive);
    require_keys_eq!(ctx.accounts.accepted_payment_token.mint, ctx.accounts.mint.key(), StoreError::PaymentTokenNotAllowed);

    let option = &mut ctx.accounts.game_payment_option;
    let is_new = option.game == Pubkey::default();
    if !is_new {
        require_keys_eq!(option.game, ctx.accounts.game.key(), StoreError::GamePaymentOptionMismatch);
    }
    let was_active = !is_new && option.active;

    option.game = ctx.accounts.game.key();
    option.mint = ctx.accounts.mint.key();
    option.base_price = base_price;
    option.active = active;
    option.bump = ctx.bumps.game_payment_option;

    // H3 guard counter: track ACTIVE options so `set_game_free_flag(true)`
    // can fail closed without enumerating PDAs (which on-chain code cannot
    // do). Checked math: an underflow here would mean corrupted state, and
    // must fail rather than silently wrap.
    let cfg = &mut ctx.accounts.game_store_config;
    if active && !was_active {
        cfg.active_payment_options = cfg
            .active_payment_options
            .checked_add(1)
            .ok_or(StoreError::MathOverflow)?;
    } else if !active && was_active {
        cfg.active_payment_options = cfg
            .active_payment_options
            .checked_sub(1)
            .ok_or(StoreError::MathOverflow)?;
    }

    emit!(GamePaymentOptionSet {
        game: option.game,
        mint: option.mint,
        base_price,
        active,
    });
    Ok(())
}
