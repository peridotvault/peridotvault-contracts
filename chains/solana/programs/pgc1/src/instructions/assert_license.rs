use anchor_lang::prelude::*;
use crate::utils::validation::validate_license;
use crate::AssertLicense;

pub fn handler(ctx: Context<AssertLicense>) -> Result<()> {
    validate_license(&ctx.accounts.license_account)?;
    Ok(())
}
