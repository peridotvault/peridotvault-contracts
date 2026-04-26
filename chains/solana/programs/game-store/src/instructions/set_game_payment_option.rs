use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GamePaymentOptionSet,
    state::{
        AcceptedPaymentToken, AuthorizedProgram, GamePaymentOption,
        GameStoreConfig, ROLE_REGISTRY,
    },
};

#[derive(Accounts)]
pub struct SetGamePaymentOption<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub publisher: Option<Signer<'info>>,

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
    pub registry_game: Account<'info, registry_program::state::RegistryGame>,
    #[account(
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

    let publisher_key = ctx.accounts.game.publisher;

    if let Some(ref publisher) = ctx.accounts.publisher {
        require_keys_eq!(publisher.key(), publisher_key, StoreError::Unauthorized);
    } else {
        require!(
            ctx.accounts.authorized_registry_program.role >= ROLE_REGISTRY,
            StoreError::InsufficientRole
        );
    }

    require_keys_eq!(ctx.accounts.registry_game.game, ctx.accounts.game.key(), StoreError::RegistryGameMismatch);
    require!(
        matches!(
            ctx.accounts.registry_game.status,
            registry_program::state::GameStatus::Active
        ),
        StoreError::GameNotActive
    );
    require!(ctx.accounts.game_store_config.active, StoreError::StoreGameInactive);
    require_keys_eq!(ctx.accounts.accepted_payment_token.mint, ctx.accounts.mint.key(), StoreError::PaymentTokenNotAllowed);

    let option = &mut ctx.accounts.game_payment_option;
    if option.game != Pubkey::default() {
        require_keys_eq!(option.game, ctx.accounts.game.key(), StoreError::GamePaymentOptionMismatch);
    }

    option.game = ctx.accounts.game.key();
    option.mint = ctx.accounts.mint.key();
    option.base_price = base_price;
    option.active = active;
    option.bump = ctx.bumps.game_payment_option;

    emit!(GamePaymentOptionSet {
        game: option.game,
        mint: option.mint,
        base_price,
        active,
    });
    Ok(())
}
