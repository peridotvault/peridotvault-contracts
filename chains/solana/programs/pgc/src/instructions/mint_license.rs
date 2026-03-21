use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{self, MintTo, Token2022},
    token_interface::{Mint, TokenAccount},
};

use crate::{
    constants::*,
    errors::Pgc1Error,
    events::LicenseMinted,
    states::{GameState, LicenseAccount, MinterAuthority},
};

#[derive(Accounts)]
pub struct MintLicense<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub game_state: Account<'info, GameState>,

    /// Receiver of license
    /// CHECK: validated by PDA/ATA relations
    pub user: UncheckedAccount<'info>,

    /// CHECK: PDA authority for minting
    #[account(
        seeds = [GAME_AUTHORITY_SEED, game_state.key().as_ref()],
        bump = game_state.authority_bump
    )]
    pub game_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [MINTER_AUTH_SEED, game_state.key().as_ref(), signer.key().as_ref()],
        bump = minter_auth.bump,
        constraint = minter_auth.is_authorized @ Pgc1Error::Unauthorized,
        constraint = minter_auth.game == game_state.key() @ Pgc1Error::Unauthorized,
        constraint = minter_auth.account == signer.key() @ Pgc1Error::Unauthorized,
    )]
    pub minter_auth: Account<'info, MinterAuthority>,

    #[account(
        mut,
        constraint = mint.key() == game_state.mint @ Pgc1Error::LicenseAccountMismatch
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        space = LicenseAccount::SPACE,
        seeds = [LICENSE_SEED, game_state.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MintLicense>, expires_at: i64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let user = ctx.accounts.user.key();
    require!(user != Pubkey::default(), Pgc1Error::InvalidReceiver);

    let game_key = ctx.accounts.game_state.key();
    let license = &mut ctx.accounts.license_account;

    let is_new = license.game == Pubkey::default();
    let mut should_mint_badge = false;

    if is_new {
        license.bump = ctx.bumps.license_account;
        license.game = game_key;
        license.user = user;
        license.issued_at = now;
        license.expires_at = expires_at;
        license.badge_minted = false;
    } else {
        require!(license.game == game_key, Pgc1Error::LicenseAccountMismatch);
        require!(license.user == user, Pgc1Error::LicenseAccountMismatch);

        let current_expires = license.expires_at;
        let current_valid = current_expires == 0 || current_expires > now;
        let incoming_permanent = expires_at == 0;
        let current_permanent = current_expires == 0;

        if !current_valid {
            license.issued_at = now;
            license.expires_at = expires_at;
        } else if current_permanent {
            // never downgrade permanent
        } else if incoming_permanent {
            license.issued_at = now;
            license.expires_at = 0;
        } else if expires_at > current_expires {
            license.issued_at = now;
            license.expires_at = expires_at;
        } else {
            // keep current
        }
    }

    if !license.badge_minted {
        let authority_seeds: &[&[u8]] = &[
            GAME_AUTHORITY_SEED,
            game_key.as_ref(),
            &[ctx.accounts.game_state.authority_bump],
        ];
        let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.game_authority.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        license.badge_minted = true;
        should_mint_badge = true;
    }

    emit!(LicenseMinted {
        game: game_key,
        user,
        issued_at: license.issued_at,
        expires_at: license.expires_at,
        minter: ctx.accounts.signer.key(),
        badge_minted: should_mint_badge,
    });

    Ok(())
}
