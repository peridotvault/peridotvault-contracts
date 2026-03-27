use anchor_lang::prelude::*;
use crate::state::LicenseAccount;
use crate::errors::PgcError;

pub fn validate_license(license: &LicenseAccount) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    if license.expires_at != 0 && license.expires_at < now {
        return Err(error!(PgcError::LicenseExpired));
    }
    Ok(())
}
