use anchor_lang::prelude::*;
use crate::PgcError;
use crate::state::*;

pub fn handler(ctx: Context<CanAccessGame>) -> Result<bool> {
    let license = &ctx.accounts.license_account;
    let now = Clock::get()?.unix_timestamp;

    if license.expires_at == 0 || license.expires_at > now {
        Ok(true)
    } else {
        err!(PgcError::LicenseExpired)
    }
}

