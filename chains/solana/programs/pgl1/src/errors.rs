use quasar_lang::prelude::*;

#[error_code]
pub enum PglError {
    Unauthorized,
    AuthorizedActorInactive,
    InsufficientCreateGameFee,
    InvalidGameId,
    InvalidMetadataUri,
    GameAlreadyExists,
    LicenseAlreadyExists,
    LicenseNotFound,
    InvalidExpiry,
    NonceOverflow,
    CreatorStateNotEmpty,
    AuthorizedActorStillActive,
}
