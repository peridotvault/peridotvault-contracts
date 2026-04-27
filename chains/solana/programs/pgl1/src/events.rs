use quasar_lang::prelude::*;

pub struct PglInitialized {
    pub authority: Address,
    pub treasury: Address,
    pub create_game_fee_lamports: u64,
}

pub struct CreateGameFeeUpdated {
    pub old_fee: u64,
    pub new_fee: u64,
}

pub struct TreasuryUpdated {
    pub authority: Address,
    pub old_treasury: Address,
    pub new_treasury: Address,
}

pub struct AuthorityUpdated {
    pub old_authority: Address,
    pub new_authority: Address,
}

pub struct AuthorizedActorAdded {
    pub actor: Address,
}

pub struct AuthorizedActorDeactivated {
    pub actor: Address,
}

pub struct AuthorizedActorClosed {
    pub actor: Address,
}

pub struct CreatorStateClosed {
    pub creator: Address,
}

pub struct GameCreated<'a> {
    pub game: Address,
    pub creator: Address,
    pub publisher: Address,
    pub nonce: u64,
    pub game_id: &'a str,
    pub metadata_uri: &'a str,
    pub created_at: i64,
}

pub struct PublisherUpdated {
    pub game: Address,
    pub old_publisher: Address,
    pub new_publisher: Address,
}

pub struct MetadataUriUpdated<'a> {
    pub game: Address,
    pub publisher: Address,
    pub old_uri: &'a str,
    pub new_uri: &'a str,
}

pub struct LicenseMinted {
    pub license: Address,
    pub holder: Address,
    pub game: Address,
    pub issued_at: i64,
    pub expires_at: Option<i64>,
}

pub struct LicenseRenewed {
    pub license: Address,
    pub holder: Address,
    pub game: Address,
    pub old_expires_at: Option<i64>,
    pub new_expires_at: i64,
}

macro_rules! impl_noop_emit {
    ($($name:ident $(<$lt:lifetime>)?),* $(,)?) => {
        $(impl$(<$lt>)? $name$(<$lt>)? {
            #[inline(always)]
            pub fn emit_log(self) -> Result<(), ProgramError> {
                Ok(())
            }
        })*
    };
}

impl_noop_emit!(
    PglInitialized,
    CreateGameFeeUpdated,
    TreasuryUpdated,
    AuthorityUpdated,
    AuthorizedActorAdded,
    AuthorizedActorDeactivated,
    AuthorizedActorClosed,
    CreatorStateClosed,
    GameCreated<'a>,
    PublisherUpdated,
    MetadataUriUpdated<'a>,
    LicenseMinted,
    LicenseRenewed,
);
