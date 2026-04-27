use crate::{
    errors::RegistryError,
    events::GameStatusUpdated,
    state::{GameStatus, RegistryConfig, RegistryGame, REGISTRY_CONFIG_SEED},
};
use quasar_lang::prelude::*;
#[derive(Accounts)]
pub struct UpdateGameStatus<'info> {
    pub authority: &'info Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump, has_one=authority)]
    pub config: &'info Account<RegistryConfig>,
    #[account(mut)]
    pub registry_game: Account<RegistryGame<'info>>,
}
pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, UpdateGameStatus<'info>>,
    status: u8,
) -> Result<(), ProgramError> {
    let next_status = GameStatus::from_u8(status).ok_or(RegistryError::InvalidStatusTransition)?;
    let current_status = ctx.accounts.registry_game.status.get();
    let valid = matches!(
        (current_status, next_status),
        (GameStatus::Active, GameStatus::Suspended)
            | (GameStatus::Suspended, GameStatus::Active)
            | (GameStatus::Active, GameStatus::Banned)
            | (GameStatus::Suspended, GameStatus::Banned)
    );
    require!(valid, RegistryError::InvalidStatusTransition);
    ctx.accounts.registry_game.status = next_status.into();
    emit!(GameStatusUpdated {
        game: ctx.accounts.registry_game.game,
        old_status: current_status.as_u8(),
        new_status: status,
        authority: *ctx.accounts.authority.address()
    })?;
    Ok(())
}
