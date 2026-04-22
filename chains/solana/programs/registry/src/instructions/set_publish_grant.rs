use anchor_lang::prelude::*;

use crate::{
    errors::RegistryError,
    events::PublishGrantSet,
    state::{PublishGrant, RegistryConfig},
};

#[derive(Accounts)]
pub struct SetPublishGrant<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    /// CHECK: publisher target for grant PDA seed
    pub publisher: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        space = PublishGrant::SPACE,
        seeds = [b"publish_grant", publisher.key().as_ref()],
        bump
    )]
    pub publish_grant: Account<'info, PublishGrant>,

    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<SetPublishGrant>, expired_at: Option<i64>) -> Result<()> {
    if let Some(ts) = expired_at {
        require!(ts > 0, RegistryError::InvalidExpiry);
    }

    let grant = &mut ctx.accounts.publish_grant;
    grant.publisher = ctx.accounts.publisher.key();
    grant.expired_at = expired_at;
    grant.bump = ctx.bumps.publish_grant;

    emit!(PublishGrantSet {
        publisher: grant.publisher,
        expired_at,
    });

    Ok(())
}
