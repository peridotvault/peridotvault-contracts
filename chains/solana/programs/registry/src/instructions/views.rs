use anchor_lang::prelude::*;

use crate::{
    constants::is_native_sol_payment_method,
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    states::{RegistrationFeeOption, RegistryGame, RegistryState},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct RegistrationFeeView {
    pub payment_method: Pubkey,
    pub amount: u64,
    pub is_native_sol: bool,
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

pub fn get_registration_fee(
    ctx: Context<GetRegistryView>,
    payment_method: Pubkey,
) -> Result<RegistrationFeeView> {
    let option = ctx
        .accounts
        .registry_state
        .registration_fee_option(&payment_method)
        .ok_or(error!(RegistryError::RegistrationFeeOptionNotFound))?;
    Ok(registration_fee_view(option))
}

pub fn get_registration_fees(ctx: Context<GetRegistryView>) -> Result<Vec<RegistrationFeeView>> {
    Ok(
        ctx.accounts
            .registry_state
            .registration_fee_options
            .iter()
            .map(registration_fee_view)
            .collect(),
    )
}

fn registration_fee_view(option: &RegistrationFeeOption) -> RegistrationFeeView {
    RegistrationFeeView {
        payment_method: option.payment_method,
        amount: option.amount,
        is_native_sol: is_native_sol_payment_method(&option.payment_method),
    }
}

pub fn is_fee_exempt(ctx: Context<GetRegistryView>, account: Pubkey) -> Result<bool> {
    Ok(ctx.accounts.registry_state.is_fee_exempt(&account))
}

pub fn is_admin(ctx: Context<GetRegistryView>, account: Pubkey) -> Result<bool> {
    Ok(ctx.accounts.registry_state.is_admin(&account))
}
