use anchor_lang::prelude::*;

use crate::{
    constants::MINTER_AUTH_SEED,
    errors::Pgc1Error,
    events::PublisherUpdated,
    states::{GameState, MinterAuthority},
};

#[derive(Accounts)]
pub struct SetPublisher<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        has_one = publisher @ Pgc1Error::Unauthorized
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: new canonical publisher
    pub new_publisher: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [MINTER_AUTH_SEED, game_state.key().as_ref(), publisher.key().as_ref()],
        bump = old_publisher_minter_auth.bump,
        constraint = old_publisher_minter_auth.game == game_state.key() @ Pgc1Error::Unauthorized,
        constraint = old_publisher_minter_auth.account == publisher.key() @ Pgc1Error::Unauthorized,
    )]
    pub old_publisher_minter_auth: Account<'info, MinterAuthority>,

    #[account(
        init_if_needed,
        payer = publisher,
        space = MinterAuthority::SPACE,
        seeds = [MINTER_AUTH_SEED, game_state.key().as_ref(), new_publisher.key().as_ref()],
        bump
    )]
    pub new_publisher_minter_auth: Account<'info, MinterAuthority>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SetPublisher>) -> Result<()> {
    let new_publisher = ctx.accounts.new_publisher.key();
    require!(
        new_publisher != Pubkey::default(),
        Pgc1Error::InvalidPublisher
    );

    let game_state = &mut ctx.accounts.game_state;
    let old_publisher = game_state.publisher;
    game_state.publisher = new_publisher;

    let old_publisher_minter_auth = &mut ctx.accounts.old_publisher_minter_auth;
    old_publisher_minter_auth.is_authorized = false;

    let new_publisher_minter_auth = &mut ctx.accounts.new_publisher_minter_auth;
    new_publisher_minter_auth.bump = ctx.bumps.new_publisher_minter_auth;
    new_publisher_minter_auth.game = game_state.key();
    new_publisher_minter_auth.account = new_publisher;
    new_publisher_minter_auth.is_authorized = true;

    emit!(PublisherUpdated {
        game: game_state.key(),
        old_publisher,
        new_publisher,
    });

    Ok(())
}
