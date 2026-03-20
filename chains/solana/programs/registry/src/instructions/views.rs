use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    states::{RegistryGame, RegistryState},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct RegistrationFeeView {
    pub amount: u64,
    pub token: Pubkey,
}

#[derive(Accounts)]
pub struct GetRegistryView<'info> {
    #[account(
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
}

pub fn get_game(ctx: Context<GetRegistryView>, game_id: String) -> Result<RegistryGame> {
    ctx.accounts
        .registry_state
        .get_game(&game_id)
        .cloned()
        .ok_or(error!(RegistryError::GameNotFound))
}

pub fn get_all_games(ctx: Context<GetRegistryView>) -> Result<Vec<RegistryGame>> {
    Ok(ctx.accounts.registry_state.games.clone())
}

pub fn get_contract_address(ctx: Context<GetRegistryView>, game_id: String) -> Result<Pubkey> {
    Ok(
        ctx.accounts
            .registry_state
            .get_game(&game_id)
            .ok_or(error!(RegistryError::GameNotFound))?
            .contract_address,
    )
}

pub fn get_status(ctx: Context<GetRegistryView>, game_id: String) -> Result<u8> {
    Ok(
        ctx.accounts
            .registry_state
            .get_game(&game_id)
            .ok_or(error!(RegistryError::GameNotFound))?
            .status,
    )
}

pub fn get_governance(ctx: Context<GetRegistryView>) -> Result<Pubkey> {
    Ok(ctx.accounts.registry_state.governance)
}

pub fn get_treasury(ctx: Context<GetRegistryView>) -> Result<Pubkey> {
    Ok(ctx.accounts.registry_state.treasury)
}

pub fn get_factory(ctx: Context<GetRegistryView>) -> Result<Pubkey> {
    Ok(ctx.accounts.registry_state.factory)
}

pub fn get_registration_fee(ctx: Context<GetRegistryView>) -> Result<RegistrationFeeView> {
    Ok(RegistrationFeeView {
        amount: ctx.accounts.registry_state.registration_fee,
        token: ctx.accounts.registry_state.registration_fee_token,
    })
}

pub fn is_fee_exempt(ctx: Context<GetRegistryView>, account: Pubkey) -> Result<bool> {
    Ok(ctx.accounts.registry_state.is_fee_exempt(&account))
}

pub fn is_admin(ctx: Context<GetRegistryView>, account: Pubkey) -> Result<bool> {
    Ok(ctx.accounts.registry_state.is_admin(&account))
}
