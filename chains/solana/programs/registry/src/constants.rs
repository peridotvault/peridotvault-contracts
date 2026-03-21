pub const REGISTRY_STATE_SEED: &[u8] = b"registry_state";

pub const MAX_GAME_ID_LEN: usize = 128;
pub const MAX_GAMES: usize = 16;
pub const MAX_ADMINS: usize = 16;
pub const MAX_FEE_EXEMPTIONS: usize = 16;

pub const STATUS_PENDING: u8 = 0;
pub const STATUS_APPROVED: u8 = 1;
pub const STATUS_BANNED: u8 = 2;

pub fn is_valid_status(status: u8) -> bool {
    matches!(status, STATUS_PENDING | STATUS_APPROVED | STATUS_BANNED)
}
