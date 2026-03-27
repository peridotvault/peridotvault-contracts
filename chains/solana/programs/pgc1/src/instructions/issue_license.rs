use anchor_lang::prelude::*;
use crate::events::LicenseIssued;
use crate::IssueLicense;

pub fn handler(ctx: Context<IssueLicense>, expires_at: i64) -> Result<()> {
    let license = &mut ctx.accounts.license_account;
    license.owner = ctx.accounts.user.key();
    license.game = ctx.accounts.game.key();
    license.issued_at = Clock::get()?.unix_timestamp;
    license.expires_at = expires_at;
    license.bump = ctx.bumps.license_account;

    emit!(LicenseIssued {
        owner: license.owner,
        game: license.game,
        expires_at,
    });

    Ok(())
}
