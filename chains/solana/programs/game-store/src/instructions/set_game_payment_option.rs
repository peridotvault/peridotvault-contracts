use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GamePaymentOptionSet,
    state::{
        AcceptedPaymentToken, AuthorizedRegistryProgram, AuthorizedSourceProgram, GamePaymentOption,
        GameStoreConfig, RegistryGameMirror, RegistryGameStatus, SourceGameMirror,
    },
};

#[derive(Accounts)]
pub struct SetGamePaymentOption<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
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
    pub mint: UncheckedAccount<'info>,
    #[account(
        constraint = accepted_payment_token.active @ StoreError::PaymentTokenDisabled,
        seeds = [b"accepted_payment_token", mint.key().as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,
    #[account(
        init_if_needed,
        payer = publisher,
        space = 8 + GamePaymentOption::LEN,
        seeds = [b"game_payment_option", game.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub game_payment_option: Account<'info, GamePaymentOption>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SetGamePaymentOption>, base_price: u64, active: bool) -> Result<()> {
    require!(base_price > 0, StoreError::InvalidPrice);
    require_keys_eq!(ctx.accounts.game.publisher, ctx.accounts.publisher.key(), StoreError::Unauthorized);
    require_keys_eq!(ctx.accounts.registry_game.game, ctx.accounts.game.key(), StoreError::RegistryGameMismatch);
    require!(matches!(ctx.accounts.registry_game.status, RegistryGameStatus::Active), StoreError::GameNotActive);
    require!(ctx.accounts.game_store_config.active, StoreError::StoreGameInactive);
    require_keys_eq!(ctx.accounts.accepted_payment_token.mint, ctx.accounts.mint.key(), StoreError::PaymentTokenNotAllowed);

    let option = &mut ctx.accounts.game_payment_option;
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
