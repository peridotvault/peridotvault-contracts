pub mod create_game;
pub mod mint_license;
pub mod set_minter;
pub mod revoke_license;
pub mod has_license;
pub mod can_access_game;
pub mod update_metadata_uri;

pub use create_game::create_game_handler;
pub use mint_license::handler as mint_license_handler;
pub use set_minter::handler as set_minter_handler;
pub use revoke_license::handler as revoke_license_handler;
pub use has_license::handler as has_license_handler;
pub use can_access_game::handler as can_access_game_handler;
pub use update_metadata_uri::handler as update_metadata_uri_handler;
