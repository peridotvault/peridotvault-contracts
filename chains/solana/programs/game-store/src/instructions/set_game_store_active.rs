use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameStoreActiveUpdated,
    state::{
        AuthorizedRegistryProgram, AuthorizedSourceProgram, GameStoreConfig, RegistryGameMirror,
        RegistryGameStatus, SourceGameMirror,
    },
};

#[derive(Accounts)]
pub struct SetGameStoreActive<'info> {
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
        mut,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
}

pub(crate) fn handler(ctx: Context<SetGameStoreActive>, active: bool) -> Result<()> {
    require_keys_eq!(ctx.accounts.game.publisher, ctx.accounts.publisher.key(), StoreError::Unauthorized);
    require_keys_eq!(ctx.accounts.registry_game.game, ctx.accounts.game.key(), StoreError::RegistryGameMismatch);
    require!(matches!(ctx.accounts.registry_game.status, RegistryGameStatus::Active), StoreError::GameNotActive);

    ctx.accounts.game_store_config.active = active;

    emit!(GameStoreActiveUpdated {
        game: ctx.accounts.game.key(),
        active,
    });
    Ok(())
}
