use anchor_lang::prelude::*;

use crate::{
    errors::{RegistryError, XchainErrorCode},
    events::{
        XchainEmergencyFreeze, XchainGameActivated, XchainGameDeprecated,
        XchainGameProposed, XchainProposalCancelled, XchainUpdateCancelled,
        XchainUpdateExecuted, XchainUpdateProposed,
    },
    state::{RegistryConfig, XchainGameEntry, XchainIdFormat, XchainPendingUpdate, XchainStatus},
    xchain,
};

fn to_logic_format(f: XchainIdFormat) -> xchain::XchainIdFormat {
    match f {
        XchainIdFormat::Evm20 => xchain::XchainIdFormat::Evm20,
        XchainIdFormat::Solana32 => xchain::XchainIdFormat::Solana32,
    }
}

fn to_logic_status(s: XchainStatus) -> xchain::XchainStatus {
    match s {
        XchainStatus::Proposed => xchain::XchainStatus::Proposed,
        XchainStatus::Active => xchain::XchainStatus::Active,
        XchainStatus::Deprecated => xchain::XchainStatus::Deprecated,
        XchainStatus::Frozen => xchain::XchainStatus::Frozen,
    }
}

fn map_err(e: xchain::XchainError) -> Error {
    match e {
        xchain::XchainError::UnknownChainId => error!(XchainErrorCode::UnknownChainId),
        xchain::XchainError::InvalidPublisher => error!(XchainErrorCode::InvalidPublisher),
        xchain::XchainError::EmptyLocalId => error!(XchainErrorCode::EmptyLocalId),
        xchain::XchainError::LocalIdTooLong => error!(XchainErrorCode::LocalIdTooLong),
        xchain::XchainError::InvalidLocalIdCharset => {
            error!(XchainErrorCode::InvalidLocalIdCharset)
        }
        xchain::XchainError::InvalidProgramId => error!(XchainErrorCode::InvalidProgramId),
        xchain::XchainError::InvalidTransition => {
            error!(XchainErrorCode::InvalidXchainTransition)
        }
        xchain::XchainError::UpdateNotEffective => error!(XchainErrorCode::UpdateNotEffective),
        xchain::XchainError::ZeroTimelock => error!(XchainErrorCode::ZeroTimelock),
        xchain::XchainError::VersionMismatch => error!(XchainErrorCode::XchainVersionMismatch),
        xchain::XchainError::UnsupportedVersion => {
            error!(XchainErrorCode::UnsupportedXchainVersion)
        }
        xchain::XchainError::PaidAmountZeroOnPaidPath => {
            error!(XchainErrorCode::PaidAmountZeroOnPaidPath)
        }
        xchain::XchainError::PaidAmountNonZeroOnFreePath => {
            error!(XchainErrorCode::PaidAmountNonZeroOnFreePath)
        }
        xchain::XchainError::PaidTokenNonEmptyOnFreePath => {
            error!(XchainErrorCode::PaidAmountNonZeroOnFreePath)
        }
        xchain::XchainError::InvalidTokenFormat => error!(XchainErrorCode::InvalidPublisher),
    }
}

#[derive(Accounts)]
#[instruction(game_id: [u8; 32])]
pub struct ProposeXchainGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    #[account(
        init,
        payer = authority,
        space = XchainGameEntry::SPACE,
        seeds = [b"m1-entry", game_id.as_ref()],
        bump
    )]
    pub entry: Account<'info, XchainGameEntry>,

    pub system_program: Program<'info, System>,
}

pub(crate) fn propose_handler(
    ctx: Context<ProposeXchainGame>,
    chain_id: String,
    publisher: Vec<u8>,
    local_id: String,
    program_id: Vec<u8>,
    format: u8,
    game_id: [u8; 32],
) -> Result<()> {
    xchain::validate_chain_id(&chain_id).map_err(map_err)?;
    xchain::validate_local_id(&local_id).map_err(map_err)?;
    let fmt = XchainIdFormat::from_u8(format).ok_or(error!(XchainErrorCode::InvalidProgramId))?;
    let logic_fmt = to_logic_format(fmt);
    xchain::validate_sized_bytes(&publisher, logic_fmt, false).map_err(map_err)?;
    xchain::validate_sized_bytes(&program_id, logic_fmt, true).map_err(map_err)?;

    // The PDA key MUST be the canonical derivation — never trust the arg blindly.
    let derived = xchain::derive_game_id(&chain_id, &publisher, &local_id);
    if derived != game_id {
        return err!(XchainErrorCode::GameIdMismatch);
    }
    let entry = &mut ctx.accounts.entry;
    entry.chain_id = chain_id.clone();
    entry.program_id = program_id;
    entry.format = fmt;
    entry.version = xchain::XCHAIN_CURRENT_VERSION;
    entry.status = XchainStatus::Proposed;
    entry.game_id = game_id;
    entry.registered_at = 0;
    entry.pending = None;

    emit!(XchainGameProposed { game_id, chain_id });
    Ok(())
}

#[derive(Accounts)]
pub struct XchainEntryAdmin<'info> {
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    #[account(
        mut,
        seeds = [b"m1-entry", entry.game_id.as_ref()],
        bump
    )]
    pub entry: Account<'info, XchainGameEntry>,
}

fn read_gate(entry: &XchainGameEntry) -> Result<()> {
    if entry.version > xchain::XCHAIN_CURRENT_VERSION {
        return err!(XchainErrorCode::UnsupportedXchainVersion);
    }
    Ok(())
}

pub(crate) fn activate_handler(ctx: Context<XchainEntryAdmin>) -> Result<()> {
    let entry = &mut ctx.accounts.entry;
    read_gate(entry)?;
    xchain::transition(to_logic_status(entry.status), xchain::XchainStatus::Active, false).map_err(map_err)?;
    entry.status = XchainStatus::Active;
    entry.registered_at = Clock::get()?.unix_timestamp;
    emit!(XchainGameActivated {
        game_id: entry.game_id,
        registered_at: entry.registered_at,
    });
    Ok(())
}

pub(crate) fn deprecate_handler(ctx: Context<XchainEntryAdmin>) -> Result<()> {
    let entry = &mut ctx.accounts.entry;
    read_gate(entry)?;
    xchain::transition(to_logic_status(entry.status), xchain::XchainStatus::Deprecated, false)
        .map_err(map_err)?;
    entry.status = XchainStatus::Deprecated;
    emit!(XchainGameDeprecated { game_id: entry.game_id });
    Ok(())
}

/// Emergency freeze: bypasses any pending timelock, immediate, distinct event.
pub(crate) fn freeze_handler(ctx: Context<XchainEntryAdmin>) -> Result<()> {
    let entry = &mut ctx.accounts.entry;
    read_gate(entry)?;
    xchain::transition(to_logic_status(entry.status), xchain::XchainStatus::Frozen, true).map_err(map_err)?;
    entry.status = XchainStatus::Frozen;
    emit!(XchainEmergencyFreeze {
        game_id: entry.game_id,
        freezer: ctx.accounts.authority.key(),
    });
    Ok(())
}

pub(crate) fn propose_update_handler(
    ctx: Context<XchainEntryAdmin>,
    new_program_id: Vec<u8>,
    timelock_delay: i64,
) -> Result<()> {
    let entry = &mut ctx.accounts.entry;
    read_gate(entry)?;
    if entry.status == XchainStatus::Frozen {
        return err!(XchainErrorCode::EntryFrozen);
    }
    if entry.status != XchainStatus::Active && entry.status != XchainStatus::Deprecated {
        return err!(XchainErrorCode::InvalidXchainTransition);
    }
    if entry.pending.is_some() {
        return err!(XchainErrorCode::UpdateAlreadyPending);
    }
    xchain::check_timelock_delay(timelock_delay).map_err(map_err)?;
    xchain::validate_sized_bytes(&new_program_id, to_logic_format(entry.format), true).map_err(map_err)?;
    let now = Clock::get()?.unix_timestamp;
    entry.pending = Some(XchainPendingUpdate {
        proposed_program_id: new_program_id,
        proposed_format: entry.format,
        proposed_at: now,
        effective_at: now.saturating_add(timelock_delay),
        timelock_delay,
        proposer: ctx.accounts.authority.key().to_bytes().to_vec(),
    });
    emit!(XchainUpdateProposed {
        game_id: entry.game_id,
        effective_at: entry.pending.as_ref().unwrap().effective_at,
        timelock_delay,
    });
    Ok(())
}

pub(crate) fn cancel_update_handler(ctx: Context<XchainEntryAdmin>) -> Result<()> {
    let entry = &mut ctx.accounts.entry;
    read_gate(entry)?;
    if entry.pending.is_none() {
        return err!(XchainErrorCode::NoPendingUpdate);
    }
    entry.pending = None;
    emit!(XchainUpdateCancelled { game_id: entry.game_id });
    Ok(())
}

pub(crate) fn execute_update_handler(ctx: Context<XchainEntryAdmin>) -> Result<()> {
    let entry = &mut ctx.accounts.entry;
    read_gate(entry)?;
    if entry.status == XchainStatus::Frozen {
        return err!(XchainErrorCode::EntryFrozen);
    }
    let pending = entry.pending.clone().ok_or(error!(XchainErrorCode::NoPendingUpdate))?;
    let now = Clock::get()?.unix_timestamp;
    xchain::check_execute(now, pending.effective_at).map_err(map_err)?;
    entry.program_id = pending.proposed_program_id;
    entry.pending = None;
    emit!(XchainUpdateExecuted { game_id: entry.game_id });
    Ok(())
}

#[derive(Accounts)]
pub struct CloseXchainProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    #[account(
        mut,
        seeds = [b"m1-entry", entry.game_id.as_ref()],
        bump,
        close = authority
    )]
    pub entry: Account<'info, XchainGameEntry>,
}

pub(crate) fn cancel_proposal_handler(ctx: Context<CloseXchainProposal>) -> Result<()> {
    let entry = &ctx.accounts.entry;
    read_gate(entry)?;
    if entry.status != XchainStatus::Proposed {
        return err!(XchainErrorCode::InvalidXchainTransition);
    }
    emit!(XchainProposalCancelled { game_id: entry.game_id });
    Ok(())
}
