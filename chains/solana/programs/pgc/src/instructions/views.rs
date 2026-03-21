use anchor_lang::{prelude::*, AccountDeserialize};

use crate::errors::Pgc1Error;
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
    /// CHECK: manually deserialized to allow "not found" => false semantics
    pub license_account: UncheckedAccount<'info>,
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
    let license = load_license(ctx.accounts.license_account.as_ref())?
        .ok_or(error!(Pgc1Error::LicenseAccountNotFound))?;
    Ok(LicensePolicyView {
        issued_at: license.issued_at,
        expires_at: license.expires_at,
    })
}

pub fn has_license(ctx: Context<GetLicenseView>) -> Result<bool> {
    let Some(license) = load_license(ctx.accounts.license_account.as_ref())? else {
        return Ok(false);
    };
    let now = Clock::get()?.unix_timestamp;
    Ok(license.expires_at == 0 || license.expires_at > now)
}

pub fn can_access_game(ctx: Context<GetLicenseView>) -> Result<bool> {
    let Some(license) = load_license(ctx.accounts.license_account.as_ref())? else {
        return Ok(false);
    };
    let now = Clock::get()?.unix_timestamp;
    Ok(license.expires_at == 0 || license.expires_at > now)
}

fn load_license(account: &AccountInfo<'_>) -> Result<Option<LicenseAccount>> {
    if account.owner != &crate::ID || account.data_is_empty() {
        return Ok(None);
    }

    let mut license_data: &[u8] = &account.data.borrow();
    let license = LicenseAccount::try_deserialize(&mut license_data)
        .map_err(|_| error!(Pgc1Error::LicenseAccountMismatch))?;
    Ok(Some(license))
}
