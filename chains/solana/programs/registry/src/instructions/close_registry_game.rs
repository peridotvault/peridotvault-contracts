use crate::{
    errors::RegistryError,
    events::GameClosed,
    external::PglGame,
    state::{GameStatus, RegistryConfig, RegistryGame, REGISTRY_CONFIG_SEED},
};
use quasar_lang::prelude::*;
#[derive(Accounts)]
pub struct CloseRegistryGame<'info> {
    pub publisher: &'info mut Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump)]
    pub config: &'info Account<RegistryConfig>,
    pub game: &'info Account<PglGame>,
    #[account(mut)]
    pub registry_game: Account<RegistryGame<'info>>,
    #[account(mut)]
    pub treasury: &'info UncheckedAccount,
}
pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, CloseRegistryGame<'info>>,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.game.publisher()?,
        *ctx.accounts.publisher.address(),
        RegistryError::Unauthorized
    );
    let status = ctx.accounts.registry_game.status.get();
    require!(
        status == GameStatus::Suspended || status == GameStatus::Banned,
        RegistryError::GameNotClosable
    );
    require_keys_eq!(
        *ctx.accounts.treasury.address(),
        ctx.accounts.config.treasury,
        RegistryError::InvalidTreasury
    );
    require_keys_eq!(
        ctx.accounts.registry_game.game,
        *ctx.accounts.game.address(),
        RegistryError::GameMismatch
    );
    let game_id = ctx.accounts.registry_game.game_id();
    emit!(GameClosed {
        game: ctx.accounts.registry_game.game,
        game_id,
        closed_by: *ctx.accounts.publisher.address()
    })?;
    ctx.accounts
        .registry_game
        .close(ctx.accounts.treasury.to_account_view())?;
    Ok(())
}
