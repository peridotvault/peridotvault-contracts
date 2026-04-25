use anchor_lang::prelude::*;

use crate::{
    errors::RegistryError,
    events::{PublishGrantCreated, PublishGrantUpdated},
    state::{PublishGrant, RegistryConfig},
};

#[derive(Accounts)]
pub struct CreatePublishGrant<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    pub publisher: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = PublishGrant::SPACE,
        seeds = [b"publish_grant", publisher.key().as_ref()],
        bump
    )]
    pub publish_grant: Account<'info, PublishGrant>,

    pub system_program: Program<'info, System>,
}

pub(crate) fn create_handler(ctx: Context<CreatePublishGrant>, expired_at: Option<i64>) -> Result<()> {
    if let Some(ts) = expired_at {
        require!(ts > Clock::get()?.unix_timestamp, RegistryError::InvalidExpiry);
    }

    let grant = &mut ctx.accounts.publish_grant;
    grant.expired_at = expired_at;
    grant.bump = ctx.bumps.publish_grant;

    emit!(PublishGrantCreated {
        publisher: ctx.accounts.publisher.key(),
        expired_at,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdatePublishGrant<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [b"publish_grant", publisher.key().as_ref()],
        bump = publish_grant.bump
    )]
    pub publish_grant: Account<'info, PublishGrant>,
}

pub(crate) fn update_handler(ctx: Context<UpdatePublishGrant>, expired_at: Option<i64>) -> Result<()> {
    if let Some(ts) = expired_at {
        require!(ts > Clock::get()?.unix_timestamp, RegistryError::InvalidExpiry);
    }

    let grant = &mut ctx.accounts.publish_grant;
    grant.expired_at = expired_at;

    emit!(PublishGrantUpdated {
        publisher: ctx.accounts.publisher.key(),
        expired_at,
    });

    Ok(())
}
