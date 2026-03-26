use anchor_lang::prelude::*;
use crate::{
    constants::{is_valid_status, GAME_REGISTRATION_SEED, REGISTRY_STATE_SEED},
    errors::RegistryError,
    events::GameStatusUpdated,
    states::{GameRegistration, RegistryState},
};

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct SetStatus<'info> {
    pub admin: Signer<'info>,

    #[account(
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,

    #[account(
        mut,
        seeds = [GAME_REGISTRATION_SEED, game_id.as_bytes()],
        bump = game_registration.bump
    )]
    pub game_registration: Account<'info, GameRegistration>,
}

pub fn handler(ctx: Context<SetStatus>, game_id: String, status: u8) -> Result<()> {
    require!(is_valid_status(status), RegistryError::InvalidStatus);

    let registry_state = &mut ctx.accounts.registry_state;
    require!(
        registry_state.is_admin(&ctx.accounts.admin.key()),
        RegistryError::Unauthorized
    );

    let game_registration = &mut ctx.accounts.game_registration;
    let old_status = game_registration.status;
    game_registration.status = status;

    emit!(GameStatusUpdated {
        game_id,
        old_status,
        new_status: status,
        admin: ctx.accounts.admin.key(),
    });

    Ok(())
}
