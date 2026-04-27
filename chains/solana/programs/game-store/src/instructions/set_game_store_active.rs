use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameStoreActiveUpdated,
    external::{assert_active_registry_game, Pgl1Program, PglGame, RegistryGame, RegistryProgram},
    state::{AuthorizedProgram, GameStoreConfig},
};

#[derive(Accounts)]
pub struct SetGameStoreActive<'info> {
    pub publisher: &'info Signer,
    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,
    pub source_program: &'info Program<Pgl1Program>,
    pub registry_program: &'info Program<RegistryProgram>,
    pub game: &'info Account<PglGame>,
    pub registry_game: &'info Account<RegistryGame>,
    #[account(
        mut,
        seeds = [b"game_store_config", game],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: &'info mut Account<GameStoreConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetGameStoreActive<'info>>,
    active: bool,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.game.publisher()?,
        *ctx.accounts.publisher.address(),
        StoreError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.registry_game.game()?,
        *ctx.accounts.game.address(),
        StoreError::RegistryGameMismatch
    );
    assert_active_registry_game(ctx.accounts.registry_game)?;

    ctx.accounts.game_store_config.active = active.into();

    emit!(GameStoreActiveUpdated {
        game: *ctx.accounts.game.address(),
        active,
    })?;
    Ok(())
}
