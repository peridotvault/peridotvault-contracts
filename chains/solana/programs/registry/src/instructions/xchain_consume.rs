//! M2 attestation consume path (ANT-487): 3-of-5 ECDSA (secp256k1)
//! EIP-712 `PGL1-XCHAIN` envelope -> verify + recognize on Solana.
//!
//! Recognition-only in v1 (no native mint — scope §"no Solana native mint").
//! Replay backstop: the recognition PDA seeds bind (game_id, licensee,
//! nonce), so re-consuming the same attestation fails at account init; a
//! fresh nonce records a fresh recognition. This is the on-chain equivalent
//! of the EVM `consumed[digest]` set.
//!
//! Solana-dest domain separation: the EIP-712 domain is DERIVED, not passed:
//! `chain_id = u64(be_bytes(keccak(dest_chain_id)[..8]))` and
//! `verifying_contract = keccak(registry_program_id)[..20]` (computed from
//! `crate::ID` at runtime). Cross-network (localnet/devnet) and
//! cross-program replay are impossible by construction — no caller-supplied
//! domain values to get wrong.
//!
//! Lenses: Access control (config authority only for init/update);
//! Checks-effects-interactions (all checks + PDA init before the event;
//! no CPI on any path); Reorg & finality (finalized-only observations +
//! config reorg delay from `observed_at`); Upgradeability (no proxy;
//! rotation is an explicit instruction).

use anchor_lang::prelude::*;

use crate::{
    errors::{RegistryError, XchainErrorCode},
    events::{XchainAttestationConsumed, XchainAttesterConfigured},
    state::{XchainAttesterConfig, XchainRecognition},
    xchain,
    xchain_attest::{
        self, AttestationView, DomainView, ATTESTER_COUNT,
    },
};

use super::xchain_entry::map_err;

/// Compact attestation signature: 64-byte secp256k1 compact sig + recovery id.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AttestationSig {
    pub sig: [u8; 64],
    pub recovery_id: u8,
}

/// Full consume arguments (every field is bound into the signed digest).
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConsumeArgs {
    pub game_id: [u8; 32],
    pub home_chain_id: String,
    pub dest_chain_id: String,
    pub licensee: [u8; 32],
    pub is_free: bool,
    pub paid_amount: u64,
    pub paid_token: Vec<u8>,
    pub paid_token_format: u8,
    pub receipt_version: u8,
    pub version: u8,
    pub nonce: u64,
    pub home_finalized_block: u64,
    pub observed_at: u64,
    pub transport_id: [u8; 32],
    pub signatures: Vec<AttestationSig>,
}

#[derive(Accounts)]
pub struct InitXchainAttester<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = XchainAttesterConfig::SPACE,
        seeds = [b"m1-attester"],
        bump
    )]
    pub config: Account<'info, XchainAttesterConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateXchainAttester<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"m1-attester"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized,
    )]
    pub config: Account<'info, XchainAttesterConfig>,
}

#[derive(Accounts)]
#[instruction(args: ConsumeArgs)]
pub struct ConsumeXchainAttestation<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(seeds = [b"m1-attester"], bump = config.bump)]
    pub config: Account<'info, XchainAttesterConfig>,

    #[account(
        init,
        payer = payer,
        space = XchainRecognition::SPACE,
        seeds = [
            b"m1-recognition",
            args.game_id.as_ref(),
            args.licensee.as_ref(),
            &args.nonce.to_le_bytes(),
        ],
        bump
    )]
    pub recognition: Account<'info, XchainRecognition>,

    pub system_program: Program<'info, System>,
}

/// Distinct non-zero attester set (mirrors EVM InvalidSignerSet).
fn check_signer_set(signers: &[[u8; 20]; ATTESTER_COUNT]) -> Result<()> {
    for (i, a) in signers.iter().enumerate() {
        if a == &[0u8; 20] {
            return Err(error!(RegistryError::Unauthorized));
        }
        for b in &signers[i + 1..] {
            if a == b {
                return Err(error!(RegistryError::Unauthorized));
            }
        }
    }
    Ok(())
}

pub(crate) fn init_handler(
    ctx: Context<InitXchainAttester>,
    signers: [[u8; 20]; ATTESTER_COUNT],
    dest_chain_id: String,
    reorg_delay: i64,
) -> Result<()> {
    check_signer_set(&signers)?;
    require!(
        dest_chain_id.len() <= crate::state::MAX_DEST_CHAIN_ID_LEN,
        XchainErrorCode::InvalidDestChainId
    );
    require!(reorg_delay >= 0, XchainErrorCode::AttestationReorgDelayNotElapsed);
    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.signers = signers;
    config.dest_chain_id = dest_chain_id;
    config.reorg_delay = reorg_delay;
    config.bump = ctx.bumps.config;

    emit!(XchainAttesterConfigured {
        reorg_delay,
        authority: config.authority,
    });
    Ok(())
}

pub(crate) fn update_handler(
    ctx: Context<UpdateXchainAttester>,
    signers: [[u8; 20]; ATTESTER_COUNT],
    dest_chain_id: String,
    reorg_delay: i64,
) -> Result<()> {
    check_signer_set(&signers)?;
    require!(
        dest_chain_id.len() <= crate::state::MAX_DEST_CHAIN_ID_LEN,
        XchainErrorCode::InvalidDestChainId
    );
    require!(reorg_delay >= 0, XchainErrorCode::AttestationReorgDelayNotElapsed);
    let config = &mut ctx.accounts.config;
    config.signers = signers;
    config.dest_chain_id = dest_chain_id;
    config.reorg_delay = reorg_delay;

    emit!(XchainAttesterConfigured {
        reorg_delay,
        authority: config.authority,
    });
    Ok(())
}

pub(crate) fn consume_handler(
    ctx: Context<ConsumeXchainAttestation>,
    args: ConsumeArgs,
) -> Result<()> {
    // Cheap policy checks first (Gas/CU lens): version lineage, payment
    // binding (AC-ENV-1..3), home-chain allowlist — before any recovery.
    xchain_attest::check_attestation_policy(
        args.version,
        args.receipt_version,
        args.is_free,
        args.paid_amount,
        &args.paid_token,
        args.paid_token_format,
        &args.home_chain_id,
    )
    .map_err(map_err)?;

    // Solana-dest domain, DERIVED (never caller-supplied).
    let dest_tag = xchain_attest::keccak256(args.dest_chain_id.as_bytes());
    let mut chain_id_bytes = [0u8; 8];
    chain_id_bytes.copy_from_slice(&dest_tag[..8]);
    let program_hash = xchain_attest::keccak256(crate::ID.as_ref());
    let mut verifier = [0u8; 20];
    verifier.copy_from_slice(&program_hash[..20]);
    let domain = DomainView {
        name: "PGL1-XCHAIN",
        version: "1",
        chain_id: u64::from_be_bytes(chain_id_bytes),
        verifying_contract: verifier,
    };
    let view = AttestationView {
        game_id: args.game_id,
        home_chain_id: &args.home_chain_id,
        dest_chain_id: &args.dest_chain_id,
        licensee: args.licensee,
        is_free: args.is_free,
        paid_amount: args.paid_amount,
        paid_token: &args.paid_token,
        paid_token_format: args.paid_token_format,
        receipt_version: args.receipt_version,
        version: args.version,
        nonce: args.nonce,
        home_finalized_block: args.home_finalized_block,
        observed_at: args.observed_at,
        transport_id: args.transport_id,
    };
    let digest = xchain_attest::attestation_digest(&domain, &view);

    // Reorg/finality policy against the Cluster clock.
    let now = Clock::get()?.unix_timestamp;
    let observed_at = i64::try_from(args.observed_at)
        .map_err(|_| map_err(xchain::XchainError::ObservedAtInFuture))?;
    xchain_attest::check_reorg(now, observed_at, ctx.accounts.config.reorg_delay)
        .map_err(map_err)?;

    // Tobias F2: dest chain is execution-anchored (config), never trusted
    // from the claim alone. Checked BEFORE recovery (cheap fail-closed).
    xchain_attest::check_dest_chain_id(
        &ctx.accounts.config.dest_chain_id,
        &args.dest_chain_id,
    )
    .map_err(map_err)?;

    // Tobias F3: cap supplied sigs at the set size before any CU spend.
    xchain_attest::check_sig_cap(args.signatures.len()).map_err(map_err)?;

    // Quorum: recover every provided sig, then require 3 DISTINCT members.
    let compact: Vec<([u8; 64], u8)> = args
        .signatures
        .iter()
        .map(|s| (s.sig, s.recovery_id))
        .collect();
    let recovered =
        xchain_attest::recover_attester_addrs(&digest, &compact).map_err(map_err)?;
    xchain_attest::verify_threshold(&recovered, &ctx.accounts.config.signers)
        .map_err(map_err)?;

    // Effects: PDA init (replay backstop) already enforced by `init`; record
    // the recognition and emit. No CPI on any path.
    let rec = &mut ctx.accounts.recognition;
    rec.game_id = args.game_id;
    rec.licensee = args.licensee;
    rec.digest = digest;
    rec.consumed_at = now;

    emit!(XchainAttestationConsumed {
        game_id: args.game_id,
        licensee: args.licensee,
        digest,
    });
    Ok(())
}
