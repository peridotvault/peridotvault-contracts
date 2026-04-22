use anchor_lang::prelude::*;

use crate::{
    errors::RegistryError,
    events::GameStatusUpdated,
    state::{GameStatus, RegistryConfig, RegistryGame},
};

#[derive(Accounts)]
pub struct UpdateGameStatus<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    #[account(
        mut,
        seeds = [b"registry_game", registry_game.game.as_ref()],
        bump = registry_game.bump
    )]
    pub registry_game: Account<'info, RegistryGame>,
}

pub(crate) fn handler(ctx: Context<UpdateGameStatus>, status: u8) -> Result<()> {
    let next_status = GameStatus::from_u8(status).ok_or(error!(RegistryError::InvalidStatusTransition))?;
    let current_status = ctx.accounts.registry_game.status;

    let valid = matches!(
        (current_status, next_status),
        (GameStatus::Active, GameStatus::Suspended)
            | (GameStatus::Suspended, GameStatus::Active)
            | (GameStatus::Active, GameStatus::Banned)
            | (GameStatus::Suspended, GameStatus::Banned)
    );

    require!(valid, RegistryError::InvalidStatusTransition);

    ctx.accounts.registry_game.status = next_status;

    emit!(GameStatusUpdated {
        game: ctx.accounts.registry_game.game,
        status,
    });

    Ok(())
}
