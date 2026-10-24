use anchor_lang::prelude::*;

#[event]
pub struct RegistryInitialized {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub pgl1_program: Pubkey,
}

#[event]
pub struct TreasuryUpdated {
    pub old_treasury: Pubkey,
    pub new_treasury: Pubkey,
}

#[event]
pub struct PaymentTokenAdded {
    pub mint: Pubkey,
    pub fee_amount: u64,
}

#[event]
pub struct PaymentTokenUpdated {
    pub mint: Pubkey,
    pub old_active: bool,
    pub new_active: bool,
    pub old_fee_amount: u64,
    pub new_fee_amount: u64,
}

#[event]
pub struct PaymentTokenRemoved {
    pub mint: Pubkey,
}

#[event]
pub struct PublishGrantCreated {
    pub publisher: Pubkey,
    pub expired_at: Option<i64>,
}

#[event]
pub struct PublishGrantUpdated {
    pub publisher: Pubkey,
    pub old_expired_at: Option<i64>,
    pub new_expired_at: Option<i64>,
}

#[event]
pub struct GameRegistered {
    pub game: Pubkey,
    pub game_id: String,
    pub publisher: Pubkey,
    pub status: u8,
    pub registered_at: i64,
}

#[event]
pub struct GameStatusUpdated {
    pub game: Pubkey,
    pub old_status: u8,
    pub new_status: u8,
    pub authority: Pubkey,
}

#[event]
pub struct GameClosed {
    pub game: Pubkey,
    pub game_id: String,
    pub closed_by: Pubkey,
}

#[event]
pub struct XchainGameProposed {
    pub game_id: [u8; 32],
    pub chain_id: String,
}

#[event]
pub struct XchainGameActivated {
    pub game_id: [u8; 32],
    pub registered_at: i64,
}

#[event]
pub struct XchainGameDeprecated {
    pub game_id: [u8; 32],
}

#[event]
pub struct XchainEmergencyFreeze {
    pub game_id: [u8; 32],
    pub freezer: Pubkey,
}

#[event]
pub struct XchainProposalCancelled {
    pub game_id: [u8; 32],
}

#[event]
pub struct XchainUpdateProposed {
    pub game_id: [u8; 32],
    pub effective_at: i64,
    pub timelock_delay: i64,
}

#[event]
pub struct XchainUpdateCancelled {
    pub game_id: [u8; 32],
}

#[event]
pub struct XchainUpdateExecuted {
    pub game_id: [u8; 32],
}

#[event]
pub struct XchainAttesterConfigured {
    pub reorg_delay: i64,
    pub authority: Pubkey,
}

#[event]
pub struct XchainAttestationConsumed {
    pub game_id: [u8; 32],
    pub licensee: [u8; 32],
    pub digest: [u8; 32],
}
