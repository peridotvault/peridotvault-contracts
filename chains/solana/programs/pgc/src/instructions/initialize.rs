use anchor_lang::prelude::*;
use anchor_lang::system_program;

use anchor_spl::{
    token_2022::{self, spl_token_2022::{extension::ExtensionType, state::Mint}, Token2022},
    token_2022_extensions::{
        metadata_pointer_initialize, non_transferable_mint_initialize, token_metadata_initialize,
        MetadataPointerInitialize, NonTransferableMintInitialize, TokenMetadataInitialize,
    },
};

use crate::{
    constants::*,
    errors::Pgc1Error,
    events::Initialized,
    states::{GameState, MinterAuthority},
};

#[derive(Accounts)]
#[instruction(game_id: String, publisher: Pubkey)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mint: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = GameState::SPACE,
        seeds = [GAME_STATE_SEED, game_id.as_bytes()],
        bump
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: PDA mint authority / metadata authority
    #[account(
        seeds = [GAME_AUTHORITY_SEED, game_state.key().as_ref()],
        bump
    )]
    pub game_authority: UncheckedAccount<'info>,

    /// CHECK: only used as PDA seed and must equal `publisher` arg
    pub publisher_account: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = MinterAuthority::SPACE,
        seeds = [MINTER_AUTH_SEED, game_state.key().as_ref(), publisher_account.key().as_ref()],
        bump
    )]
    pub publisher_minter_auth: Account<'info, MinterAuthority>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    game_id: String,
    publisher: Pubkey,
    metadata_uri: String,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), Pgc1Error::EmptyGameId);
    require!(!metadata_uri.trim().is_empty(), Pgc1Error::EmptyMetadataUri);
    require!(game_id.len() <= MAX_GAME_ID_LEN, Pgc1Error::StringTooLong);
    require!(
        metadata_uri.len() <= MAX_METADATA_URI_LEN,
        Pgc1Error::StringTooLong
    );
    require!(publisher != Pubkey::default(), Pgc1Error::InvalidPublisher);
    require!(
        ctx.accounts.publisher_account.key() == publisher,
        Pgc1Error::InvalidPublisher
    );

    let game_state = &mut ctx.accounts.game_state;
    game_state.bump = ctx.bumps.game_state;
    game_state.authority_bump = ctx.bumps.game_authority;
    game_state.mint = ctx.accounts.mint.key();
    game_state.game_id = game_id.clone();
    game_state.publisher = publisher;
    game_state.metadata_uri = metadata_uri.clone();

    let publisher_minter_auth = &mut ctx.accounts.publisher_minter_auth;
    publisher_minter_auth.bump = ctx.bumps.publisher_minter_auth;
    publisher_minter_auth.game = game_state.key();
    publisher_minter_auth.account = publisher;
    publisher_minter_auth.is_authorized = true;

    let mint_space = ExtensionType::try_calculate_account_len::<Mint>(&[
        ExtensionType::NonTransferable,
        ExtensionType::MetadataPointer,
    ])
    .map_err(|_| error!(Pgc1Error::StringTooLong))?;

    let metadata_extra_space: usize = 256;
    let total_space = mint_space + metadata_extra_space;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(total_space);

    system_program::create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.mint.to_account_info(),
            },
        ),
        lamports,
        mint_space as u64,
        &ctx.accounts.token_program.key(),
    )?;

    metadata_pointer_initialize(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(ctx.accounts.game_authority.key()),
        Some(ctx.accounts.mint.key()),
    )?;

    non_transferable_mint_initialize(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        NonTransferableMintInitialize {
            token_program_id: ctx.accounts.token_program.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        },
    ))?;

    token_2022::initialize_mint2(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_2022::InitializeMint2 {
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        0,
        &ctx.accounts.game_authority.key(),
        Some(&ctx.accounts.game_authority.key()),
    )?;

    let game_key = game_state.key();
    let authority_seeds: &[&[u8]] = &[
        GAME_AUTHORITY_SEED,
        game_key.as_ref(),
        &[game_state.authority_bump],
    ];
    let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

    token_metadata_initialize(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TokenMetadataInitialize {
                program_id: ctx.accounts.token_program.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.game_authority.to_account_info(),
                mint_authority: ctx.accounts.game_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds,
        ),
        game_id.clone(),
        BADGE_SYMBOL.to_string(),
        metadata_uri.clone(),
    )?;

    emit!(Initialized {
        game: game_state.key(),
        publisher,
        initial_minter: publisher,
        mint: ctx.accounts.mint.key(),
        game_id,
        metadata_uri,
    });

    Ok(())
}
