use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke;
use crate::state::*;
use crate::errors::PgcError;
use crate::utils::entitlement::check_entitlement;
use crate::constants::*;

// --- Create Game ---

pub fn create_game_handler(
    ctx: Context<CreateGame>,
    game_id: String,
    metadata_uri: String,
    initial_minter: Pubkey,
    price: u64,
    currency: Pubkey,
) -> Result<()> {
    // 1. Initialize Game Account
    let game_account = &mut ctx.accounts.game_account;
    game_account.game_id = game_id.clone();
    game_account.publisher = ctx.accounts.publisher.key();
    game_account.metadata_uri = metadata_uri;
    game_account.created_at = Clock::get()?.unix_timestamp;
    game_account.bump = ctx.bumps.game_account;

    game_account.exit(ctx.program_id)?;

    // 2. Initialize Initial Minter
    let minter_account = &mut ctx.accounts.initial_minter_account;
    minter_account.game = game_account.key();
    minter_account.account = initial_minter;
    minter_account.is_authorized = true;
    minter_account.bump = ctx.bumps.initial_minter_account;

    // 3. Manual CPI to Registry.register_game
    let mut reg_data = vec![122, 44, 95, 58, 89, 33, 40, 59];
    let game_id_bytes = game_id.as_bytes();
    reg_data.extend_from_slice(&(game_id_bytes.len() as u32).to_le_bytes());
    reg_data.extend_from_slice(game_id_bytes);
    reg_data.extend_from_slice(crate::ID.as_ref());
    reg_data.extend_from_slice(game_account.key().as_ref());

    let reg_ix = Instruction {
        program_id: ctx.accounts.registry_program.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.publisher.key(), true),
            AccountMeta::new_readonly(ctx.accounts.registry_config.key(), false),
            AccountMeta::new(ctx.accounts.registry_treasury.key(), false),
            AccountMeta::new(ctx.accounts.registry_game.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ],
        data: reg_data,
    };

    invoke(
        &reg_ix,
        &[
            ctx.accounts.publisher.to_account_info(),
            ctx.accounts.registry_config.to_account_info(),
            ctx.accounts.registry_treasury.to_account_info(),
            ctx.accounts.registry_game.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // 4. Manual CPI to Store.set_price
    let mut store_data = vec![16, 19, 182, 8, 149, 83, 72, 181];
    store_data.extend_from_slice(&price.to_le_bytes());
    store_data.extend_from_slice(currency.as_ref());

    let store_ix = Instruction {
        program_id: ctx.accounts.store_program.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.publisher.key(), true),
            AccountMeta::new_readonly(game_account.key(), false),
            AccountMeta::new(ctx.accounts.price_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ],
        data: store_data,
    };

    invoke(
        &store_ix,
        &[
            ctx.accounts.publisher.to_account_info(),
            game_account.to_account_info(),
            ctx.accounts.price_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    emit!(crate::events::GameCreated {
        game_id,
        publisher: ctx.accounts.publisher.key(),
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: String, metadata_uri: String, initial_minter: Pubkey)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        init,
        payer = publisher,
        space = PgcGameAccount::SPACE,
        seeds = [SEED_GAME, game_id.as_bytes()],
        bump
    )]
    pub game_account: Account<'info, PgcGameAccount>,

    #[account(
        init,
        payer = publisher,
        space = MinterAccount::SPACE,
        seeds = [SEED_MINTER, game_account.key().as_ref(), initial_minter.as_ref()],
        bump
    )]
    pub initial_minter_account: Account<'info, MinterAccount>,

    /// CHECK: Registry program
    pub registry_program: UncheckedAccount<'info>,
    /// CHECK: Store program
    pub store_program: UncheckedAccount<'info>,
    /// CHECK: Registry config PDA
    pub registry_config: UncheckedAccount<'info>,
    /// CHECK: Registry treasury
    #[account(mut)]
    pub registry_treasury: UncheckedAccount<'info>,
    /// CHECK: Registry game PDA
    #[account(mut)]
    pub registry_game: UncheckedAccount<'info>,
    /// CHECK: Store price PDA
    #[account(mut)]
    pub price_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

// --- Mint License ---

pub fn mint_license_handler(ctx: Context<MintLicense>, expires_at: i64) -> Result<()> {
    let license = &mut ctx.accounts.license_account;
    
    if license.owner == Pubkey::default() {
        license.owner = ctx.accounts.user.key();
        license.game = ctx.accounts.game.key();
        license.issued_at = Clock::get()?.unix_timestamp;
        license.expires_at = expires_at;
        license.bump = ctx.bumps.license_account;
    } else {
        license.expires_at = check_entitlement(license.expires_at, expires_at);
        license.issued_at = Clock::get()?.unix_timestamp;
    }

    emit!(crate::events::LicenseIssued {
        owner: license.owner,
        game: license.game,
        expires_at: license.expires_at,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MintLicense<'info> {
    pub minter: Signer<'info>,

    #[account(
        seeds = [SEED_MINTER, game.key().as_ref(), minter.key().as_ref()],
        bump,
        constraint = minter_account.is_authorized @ PgcError::Unauthorized
    )]
    pub minter_account: Account<'info, MinterAccount>,

    pub game: Account<'info, PgcGameAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = LicenseAccount::SPACE,
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,

    pub system_program: Program<'info, System>,
}

// --- Set Minter ---

pub fn set_minter_handler(ctx: Context<SetMinter>, account: Pubkey, is_authorized: bool) -> Result<()> {
    let minter_account = &mut ctx.accounts.minter_account;
    minter_account.game = ctx.accounts.game_account.key();
    minter_account.account = account;
    minter_account.is_authorized = is_authorized;
    minter_account.bump = ctx.bumps.minter_account;

    emit!(crate::events::MinterUpdated {
        game: minter_account.game,
        minter: account,
        is_authorized,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(account: Pubkey)]
pub struct SetMinter<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        has_one = publisher @ PgcError::Unauthorized
    )]
    pub game_account: Account<'info, PgcGameAccount>,

    #[account(
        init_if_needed,
        payer = publisher,
        space = MinterAccount::SPACE,
        seeds = [SEED_MINTER, game_account.key().as_ref(), account.as_ref()],
        bump
    )]
    pub minter_account: Account<'info, MinterAccount>,

    pub system_program: Program<'info, System>,
}

// --- Revoke License ---

pub fn revoke_license_handler(ctx: Context<RevokeLicense>) -> Result<()> {
    let license = &mut ctx.accounts.license_account;
    license.expires_at = Clock::get()?.unix_timestamp;
    
    emit!(crate::events::LicenseRevoked {
        owner: license.owner,
        game: license.game,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RevokeLicense<'info> {
    pub minter: Signer<'info>,

    #[account(
        seeds = [SEED_MINTER, game.key().as_ref(), minter.key().as_ref()],
        bump,
        constraint = minter_account.is_authorized @ PgcError::Unauthorized
    )]
    pub minter_account: Account<'info, MinterAccount>,

    pub game: Account<'info, PgcGameAccount>,

    /// CHECK: The user to revoke
    pub user: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,
}

// --- Update Metadata URI ---

pub fn update_metadata_uri_handler(ctx: Context<UpdateMetadataUri>, new_uri: String) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    game_account.metadata_uri = new_uri.clone();
    
    emit!(crate::events::MetadataUpdated {
        game: game_account.key(),
        new_uri,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateMetadataUri<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        has_one = publisher @ PgcError::Unauthorized
    )]
    pub game_account: Account<'info, PgcGameAccount>,
}

// --- Set Publisher ---

pub fn set_publisher_handler(ctx: Context<SetPublisher>, new_publisher: Pubkey) -> Result<()> {
    let game_account = &mut ctx.accounts.game_account;
    let old_publisher = game_account.publisher;
    game_account.publisher = new_publisher;
    
    emit!(crate::events::PublisherUpdated {
        game: game_account.key(),
        old_publisher,
        new_publisher,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct SetPublisher<'info> {
    #[account(
        mut,
        has_one = publisher @ PgcError::Unauthorized
    )]
    pub game_account: Account<'info, PgcGameAccount>,
    pub publisher: Signer<'info>,
}

// --- Has License ---

pub fn has_license_handler(ctx: Context<HasLicense>) -> Result<bool> {
    let license = &ctx.accounts.license_account;
    let now = Clock::get()?.unix_timestamp;

    if license.expires_at == 0 || license.expires_at > now {
        Ok(true)
    } else {
        err!(PgcError::LicenseExpired)
    }
}

#[derive(Accounts)]
pub struct HasLicense<'info> {
    pub game: Account<'info, PgcGameAccount>,
    /// CHECK: The user
    pub user: UncheckedAccount<'info>,
    #[account(
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,
}

// --- Can Access Game ---

pub fn can_access_game_handler(ctx: Context<CanAccessGame>) -> Result<bool> {
    let license = &ctx.accounts.license_account;
    let now = Clock::get()?.unix_timestamp;

    if license.expires_at == 0 || license.expires_at > now {
        Ok(true)
    } else {
        err!(PgcError::LicenseExpired)
    }
}

#[derive(Accounts)]
pub struct CanAccessGame<'info> {
    pub game: Account<'info, PgcGameAccount>,
    /// CHECK: The user
    pub user: UncheckedAccount<'info>,
    #[account(
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,
}
