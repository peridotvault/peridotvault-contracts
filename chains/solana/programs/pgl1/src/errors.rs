use anchor_lang::prelude::*;

#[error_code]
pub enum PglError {
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Authorized actor is inactive")]
    AuthorizedActorInactive,

    #[msg("Insufficient create game fee")]
    InsufficientCreateGameFee,

    #[msg("Invalid game id")]
    InvalidGameId,

    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,

    #[msg("Game already exists")]
    GameAlreadyExists,

    #[msg("License already exists")]
    LicenseAlreadyExists,

    #[msg("License not found")]
    LicenseNotFound,

    #[msg("Invalid expiry")]
    InvalidExpiry,

    #[msg("Creator nonce overflow")]
    NonceOverflow,

    #[msg("Creator state still has pending games")]
    CreatorStateNotEmpty,

    #[msg("Authorized actor is still active")]
    AuthorizedActorStillActive,

    #[msg("Missing actor signature (H2: mint requires authorized signer)")]
    MissingActorSignature,

    #[msg("Missing renewal signer (H-2: license holder must sign)")]
    MissingRenewalSigner,

    #[msg("Unsupported account version")]
    UnsupportedVersion,
}
