use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameStoreConfigInitialized,
    state::{
        AuthorizedRegistryProgram, AuthorizedSourceProgram, GameStoreConfig,
    },
};

#[derive(Accounts)]
pub struct InitGameStoreConfig<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_source_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedSourceProgram>,
    pub source_program: Program<'info, pgl1::program::Pgl1>,
    #[account(
        constraint = authorized_registry_program.active @ StoreError::RegistryProgramNotAuthorized,
        seeds = [b"authorized_registry_program", registry_program.key().as_ref()],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: Account<'info, AuthorizedRegistryProgram>,
    pub registry_program: Program<'info, registry_program::program::Registry>,
    pub game: Account<'info, pgl1::state::Game>,
    pub registry_game: Account<'info, registry_program::state::RegistryGame>,
    #[account(
        init,
        payer = publisher,
        space = GameStoreConfig::SPACE,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<InitGameStoreConfig>, active: bool) -> Result<()> {
    require_keys_eq!(ctx.accounts.game.publisher, ctx.accounts.publisher.key(), StoreError::Unauthorized);
    require_keys_eq!(ctx.accounts.registry_game.game, ctx.accounts.game.key(), StoreError::RegistryGameMismatch);
    require!(
        matches!(
            ctx.accounts.registry_game.status,
            registry_program::state::GameStatus::Active
        ),
        StoreError::GameNotActive
    );

    let cfg = &mut ctx.accounts.game_store_config;
    cfg.game = ctx.accounts.game.key();
    cfg.active = active;
    cfg.referral_bps = None;
    cfg.discount_bps = None;
    cfg.discount_starts_at = None;
    cfg.discount_expires_at = None;
    cfg.bump = ctx.bumps.game_store_config;

    emit!(GameStoreConfigInitialized {
        game: cfg.game,
        active,
    });
    Ok(())
}
