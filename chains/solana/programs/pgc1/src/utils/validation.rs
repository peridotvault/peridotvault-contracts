use anchor_lang::prelude::*;
use crate::state::LicenseAccount;
use crate::errors::PgcError;

pub fn validate_license(license: &LicenseAccount) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    
    // Check if license is permanent (expires_at == 0) or active (expires_at > now)
    if license.expires_at != 0 && license.expires_at <= now {
        return err!(PgcError::LicenseExpired);
    }
    
    Ok(())
}
