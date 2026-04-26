use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameStoreActiveUpdated,
    state::{
        AuthorizedProgram, GameStoreConfig,
    },
};

#[derive(Accounts)]
pub struct SetGameStoreActive<'info> {
    pub publisher: Signer<'info>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedProgram>,
    pub source_program: Program<'info, pgl1::program::Pgl1>,
    pub registry_program: Program<'info, registry_program::program::Registry>,
    pub game: Account<'info, pgl1::state::Game>,
    pub registry_game: Account<'info, registry_program::state::RegistryGame>,
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
    require!(
        matches!(
            ctx.accounts.registry_game.status,
            registry_program::state::GameStatus::Active
        ),
        StoreError::GameNotActive
    );

    ctx.accounts.game_store_config.active = active;

    emit!(GameStoreActiveUpdated {
        game: ctx.accounts.game.key(),
        active,
    });
    Ok(())
}
