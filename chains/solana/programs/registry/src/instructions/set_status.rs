use anchor_lang::prelude::*;

use crate::{
    constants::{is_valid_status, REGISTRY_STATE_SEED},
    errors::RegistryError,
    events::GameStatusUpdated,
    states::RegistryState,
};

#[derive(Accounts)]
pub struct SetStatus<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
}

pub fn handler(ctx: Context<SetStatus>, game_id: String, status: u8) -> Result<()> {
    require!(is_valid_status(status), RegistryError::InvalidStatus);

    let registry_state = &mut ctx.accounts.registry_state;
    require!(
        registry_state.is_admin(&ctx.accounts.admin.key()),
        RegistryError::Unauthorized
    );

    let game_index = registry_state
        .game_index(&game_id)
        .ok_or(error!(RegistryError::GameNotFound))?;
    let old_status = registry_state.games[game_index].status;
    registry_state.games[game_index].status = status;

    emit!(GameStatusUpdated {
        game_id,
        old_status,
        new_status: status,
        admin: ctx.accounts.admin.key(),
    });

    Ok(())
}
