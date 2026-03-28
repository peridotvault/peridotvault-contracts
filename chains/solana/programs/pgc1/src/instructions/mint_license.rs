use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::entitlement::check_entitlement;

pub fn handler(ctx: Context<MintLicense>, expires_at: i64) -> Result<()> {
    let license = &mut ctx.accounts.license_account;
    
    // If it's a new account (init_if_needed)
    if license.owner == Pubkey::default() {
        license.owner = ctx.accounts.user.key();
        license.game = ctx.accounts.game.key();
        license.issued_at = Clock::get()?.unix_timestamp;
        license.expires_at = expires_at;
        license.bump = ctx.bumps.license_account;
    } else {
        // Apply entitlement logic
        license.expires_at = check_entitlement(license.expires_at, expires_at);
        license.issued_at = Clock::get()?.unix_timestamp;
    }

    Ok(())
}

