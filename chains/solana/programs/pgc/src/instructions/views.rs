use anchor_lang::prelude::*;

use crate::states::{GameState, LicenseAccount, MinterAuthority};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LicensePolicyView {
    pub issued_at: i64,
    pub expires_at: i64,
}

#[derive(Accounts)]
pub struct GetGameStateView<'info> {
    pub game_state: Account<'info, GameState>,
}

#[derive(Accounts)]
pub struct GetMinterView<'info> {
    pub minter_auth: Account<'info, MinterAuthority>,
}

#[derive(Accounts)]
pub struct GetLicenseView<'info> {
    pub license_account: Account<'info, LicenseAccount>,
}

pub fn get_publisher(ctx: Context<GetGameStateView>) -> Result<Pubkey> {
    Ok(ctx.accounts.game_state.publisher)
}

pub fn get_game_id(ctx: Context<GetGameStateView>) -> Result<String> {
    Ok(ctx.accounts.game_state.game_id.clone())
}

pub fn get_metadata_uri(ctx: Context<GetGameStateView>) -> Result<String> {
    Ok(ctx.accounts.game_state.metadata_uri.clone())
}

pub fn is_minter(ctx: Context<GetMinterView>) -> Result<bool> {
    Ok(ctx.accounts.minter_auth.is_authorized)
}

pub fn get_license_policy(ctx: Context<GetLicenseView>) -> Result<LicensePolicyView> {
    let license = &ctx.accounts.license_account;
    Ok(LicensePolicyView {
        issued_at: license.issued_at,
        expires_at: license.expires_at,
    })
}

pub fn has_license(ctx: Context<GetLicenseView>) -> Result<bool> {
    let now = Clock::get()?.unix_timestamp;
    let license = &ctx.accounts.license_account;
    Ok(license.expires_at == 0 || license.expires_at > now)
}

pub fn can_access_game(ctx: Context<GetLicenseView>) -> Result<bool> {
    let now = Clock::get()?.unix_timestamp;
    let license = &ctx.accounts.license_account;
    Ok(license.expires_at == 0 || license.expires_at > now)
}
