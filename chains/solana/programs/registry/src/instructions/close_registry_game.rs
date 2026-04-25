use anchor_lang::prelude::*;

use crate::{
    errors::RegistryError,
    events::GameClosed,
    state::{GameStatus, RegistryConfig, RegistryGame},
};

#[derive(Accounts)]
pub struct CloseRegistryGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump
    )]
    pub config: Account<'info, RegistryConfig>,

    #[account(
        mut,
        seeds = [b"registry_game", registry_game.game.as_ref()],
        bump = registry_game.bump,
        close = treasury
    )]
    pub registry_game: Account<'info, RegistryGame>,

    /// CHECK: receives rent SOL from closed registry game (treasury address from config)
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
}

pub(crate) fn handler(ctx: Context<CloseRegistryGame>) -> Result<()> {
    require!(
        ctx.accounts.registry_game.status == GameStatus::Suspended
            || ctx.accounts.registry_game.status == GameStatus::Banned,
        RegistryError::GameNotClosable
    );

    require_keys_eq!(
        ctx.accounts.treasury.key(),
        ctx.accounts.config.treasury,
        RegistryError::InvalidTreasury
    );

    emit!(GameClosed {
        game: ctx.accounts.registry_game.game,
        game_id: ctx.accounts.registry_game.game_id.clone(),
    });

    Ok(())
}
