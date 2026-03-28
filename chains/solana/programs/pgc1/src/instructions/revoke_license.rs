use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<RevokeLicense>) -> Result<()> {
    // Basic revocation: set expiresAt to now.
    let license = &mut ctx.accounts.license_account;
    license.expires_at = Clock::get()?.unix_timestamp;
    
    Ok(())
}

