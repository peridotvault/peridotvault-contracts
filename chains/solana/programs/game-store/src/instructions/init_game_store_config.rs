use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameStoreConfigInitialized,
    external::{assert_active_registry_game, Pgl1Program, PglGame, RegistryGame, RegistryProgram},
    state::{AuthorizedProgram, GameStoreConfig, OptionI64, OptionU16, ROLE_REGISTRY},
};

#[derive(Accounts)]
pub struct InitGameStoreConfig<'info> {
    pub payer: &'info mut Signer,
    pub publisher: Option<&'info Signer>,

    pub source_program: &'info Program<Pgl1Program>,
    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        constraint = authorized_source_program.role == 0 @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,

    pub registry_program: &'info Program<RegistryProgram>,
    #[account(
        constraint = authorized_registry_program.active.get() @ StoreError::RegistryProgramNotAuthorized,
        constraint = authorized_registry_program.role >= ROLE_REGISTRY @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", registry_program],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: &'info Account<AuthorizedProgram>,

    pub game: &'info Account<PglGame>,
    pub registry_game: &'info Account<RegistryGame>,

    #[account(
        init,
        payer = payer,
        space = <GameStoreConfig as Space>::SPACE,
        seeds = [b"game_store_config", game],
        bump
    )]
    pub game_store_config: &'info mut Account<GameStoreConfig>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, InitGameStoreConfig<'info>>,
    active: bool,
) -> Result<(), ProgramError> {
    let publisher_key = ctx.accounts.game.publisher()?;

    if let Some(publisher) = ctx.accounts.publisher {
        require_keys_eq!(
            *publisher.address(),
            publisher_key,
            StoreError::Unauthorized
        );
    } else {
        require!(
            ctx.accounts.authorized_registry_program.role >= ROLE_REGISTRY,
            StoreError::InsufficientRole
        );
    }

    require_keys_eq!(
        ctx.accounts.registry_game.game()?,
        *ctx.accounts.game.address(),
        StoreError::RegistryGameMismatch
    );
    assert_active_registry_game(ctx.accounts.registry_game)?;

    ctx.accounts.game_store_config.set_inner(
        *ctx.accounts.game.address(),
        active,
        OptionU16::NONE,
        OptionU16::NONE,
        OptionI64::NONE,
        OptionI64::NONE,
        ctx.bumps.game_store_config,
    );

    emit!(GameStoreConfigInitialized {
        game: *ctx.accounts.game.address(),
        active,
    })?;
    Ok(())
}
