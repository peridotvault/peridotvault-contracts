use crate::{
    errors::RegistryError,
    events::{PublishGrantCreated, PublishGrantUpdated},
    instructions::read_option_i64,
    state::{PublishGrant, RegistryConfig, PUBLISH_GRANT_SEED, REGISTRY_CONFIG_SEED},
};
use quasar_lang::{prelude::*, sysvars::Sysvar};
#[derive(Accounts)]
pub struct CreatePublishGrant<'info> {
    pub authority: &'info mut Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump, has_one=authority)]
    pub config: &'info Account<RegistryConfig>,
    pub publisher: &'info Signer,
    #[account(init, payer=authority, space=<PublishGrant as Space>::SPACE, seeds=[PUBLISH_GRANT_SEED, publisher], bump)]
    pub publish_grant: &'info mut Account<PublishGrant>,
    pub system_program: &'info Program<System>,
}
pub(crate) fn create_handler<'info>(
    ctx: &mut Ctx<'info, CreatePublishGrant<'info>>,
) -> Result<(), ProgramError> {
    let mut offset = 0;
    let expired_at = read_option_i64(ctx.data, &mut offset)?;
    if let Some(ts) = expired_at {
        require!(
            ts > Clock::get()?.unix_timestamp.get(),
            RegistryError::InvalidExpiry
        );
    }
    ctx.accounts
        .publish_grant
        .set_inner(expired_at.into(), ctx.bumps.publish_grant);
    emit!(PublishGrantCreated {
        publisher: *ctx.accounts.publisher.address(),
        expired_at
    })?;
    Ok(())
}
#[derive(Accounts)]
pub struct UpdatePublishGrant<'info> {
    pub authority: &'info Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump, has_one=authority)]
    pub config: &'info Account<RegistryConfig>,
    pub publisher: &'info Signer,
    #[account(mut, seeds=[PUBLISH_GRANT_SEED, publisher], bump=publish_grant.bump)]
    pub publish_grant: &'info mut Account<PublishGrant>,
}
pub(crate) fn update_handler<'info>(
    ctx: &mut Ctx<'info, UpdatePublishGrant<'info>>,
) -> Result<(), ProgramError> {
    let mut offset = 0;
    let expired_at = read_option_i64(ctx.data, &mut offset)?;
    if let Some(ts) = expired_at {
        require!(
            ts > Clock::get()?.unix_timestamp.get(),
            RegistryError::InvalidExpiry
        );
    }
    let old_expired_at = ctx.accounts.publish_grant.expired_at.get();
    ctx.accounts.publish_grant.expired_at = expired_at.into();
    emit!(PublishGrantUpdated {
        publisher: *ctx.accounts.publisher.address(),
        old_expired_at,
        new_expired_at: expired_at
    })?;
    Ok(())
}
