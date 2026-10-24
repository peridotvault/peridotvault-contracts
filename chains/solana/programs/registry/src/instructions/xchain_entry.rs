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

pub(crate) fn map_err(e: xchain::XchainError) -> Error {
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
            error!(XchainErrorCode::PaidTokenNonEmptyOnFreePath)
        }
        xchain::XchainError::InvalidTokenFormat => {
            error!(XchainErrorCode::InvalidTokenFormat)
        }
        xchain::XchainError::UnknownAttester => error!(XchainErrorCode::UnknownAttester),
        xchain::XchainError::DuplicateAttesterSignature => {
            error!(XchainErrorCode::DuplicateAttesterSignature)
        }
        xchain::XchainError::ThresholdNotMet => {
            error!(XchainErrorCode::AttestationThresholdNotMet)
        }
        xchain::XchainError::AlreadyConsumed => {
            error!(XchainErrorCode::AttestationAlreadyConsumed)
        }
        xchain::XchainError::InvalidAttestationSignature => {
            error!(XchainErrorCode::InvalidAttestationSignature)
        }
        xchain::XchainError::ReorgDelayNotElapsed => {
            error!(XchainErrorCode::AttestationReorgDelayNotElapsed)
        }
        xchain::XchainError::ObservedAtInFuture => {
            error!(XchainErrorCode::AttestationObservedAtInFuture)
        }
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

#[cfg(test)]
mod mapping_tests {
    use super::*;
    use std::collections::HashSet;

    /// Tobias ANT-491 fix 1: every xchain logic error maps 1:1 to a DISTINCT
    /// on-chain code (no aliasing — PaidTokenNonEmptyOnFreePath and
    /// InvalidTokenFormat previously collapsed onto other variants).
    #[test]
    fn xchain_error_maps_one_to_one_distinct_codes() {
        let cases: [(xchain::XchainError, XchainErrorCode); 22] = [
            (xchain::XchainError::UnknownChainId, XchainErrorCode::UnknownChainId),
            (xchain::XchainError::InvalidPublisher, XchainErrorCode::InvalidPublisher),
            (xchain::XchainError::EmptyLocalId, XchainErrorCode::EmptyLocalId),
            (xchain::XchainError::LocalIdTooLong, XchainErrorCode::LocalIdTooLong),
            (xchain::XchainError::InvalidLocalIdCharset, XchainErrorCode::InvalidLocalIdCharset),
            (xchain::XchainError::InvalidProgramId, XchainErrorCode::InvalidProgramId),
            (xchain::XchainError::InvalidTransition, XchainErrorCode::InvalidXchainTransition),
            (xchain::XchainError::UpdateNotEffective, XchainErrorCode::UpdateNotEffective),
            (xchain::XchainError::ZeroTimelock, XchainErrorCode::ZeroTimelock),
            (xchain::XchainError::VersionMismatch, XchainErrorCode::XchainVersionMismatch),
            (xchain::XchainError::UnsupportedVersion, XchainErrorCode::UnsupportedXchainVersion),
            (xchain::XchainError::PaidAmountZeroOnPaidPath, XchainErrorCode::PaidAmountZeroOnPaidPath),
            (
                xchain::XchainError::PaidAmountNonZeroOnFreePath,
                XchainErrorCode::PaidAmountNonZeroOnFreePath,
            ),
            (
                xchain::XchainError::PaidTokenNonEmptyOnFreePath,
                XchainErrorCode::PaidTokenNonEmptyOnFreePath,
            ),
            (xchain::XchainError::InvalidTokenFormat, XchainErrorCode::InvalidTokenFormat),
            // M2 attestation (ANT-487): 1:1, same no-aliasing rule.
            (xchain::XchainError::UnknownAttester, XchainErrorCode::UnknownAttester),
            (
                xchain::XchainError::DuplicateAttesterSignature,
                XchainErrorCode::DuplicateAttesterSignature,
            ),
            (
                xchain::XchainError::ThresholdNotMet,
                XchainErrorCode::AttestationThresholdNotMet,
            ),
            (
                xchain::XchainError::AlreadyConsumed,
                XchainErrorCode::AttestationAlreadyConsumed,
            ),
            (
                xchain::XchainError::InvalidAttestationSignature,
                XchainErrorCode::InvalidAttestationSignature,
            ),
            (
                xchain::XchainError::ReorgDelayNotElapsed,
                XchainErrorCode::AttestationReorgDelayNotElapsed,
            ),
            (
                xchain::XchainError::ObservedAtInFuture,
                XchainErrorCode::AttestationObservedAtInFuture,
            ),
        ];
        let mut seen = HashSet::new();
        for (logic, code) in cases {
            let got: u32 = match map_err(logic) {
                anchor_lang::error::Error::AnchorError(a) => a.error_code_number,
                anchor_lang::error::Error::ProgramError(_) => {
                    panic!("expected AnchorError")
                }
            };
            let want: u32 = code.into();
            assert_eq!(got, want, "logic error maps to its distinct on-chain code");
            assert!(seen.insert(got), "duplicate on-chain code {}", got);
        }
    }
}
