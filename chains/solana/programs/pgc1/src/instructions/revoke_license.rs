use anchor_lang::prelude::*;
use crate::events::LicenseRevoked;
use crate::RevokeLicense;

pub fn handler(ctx: Context<RevokeLicense>) -> Result<()> {
    emit!(LicenseRevoked {
        owner: ctx.accounts.license_account.owner,
        game: ctx.accounts.game.key(),
    });
    Ok(())
}
