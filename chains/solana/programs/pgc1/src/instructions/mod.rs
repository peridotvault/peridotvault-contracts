pub mod create_game;
pub mod issue_license;
pub mod revoke_license;
pub mod assert_license;

pub use create_game::handler as create_game_handler;
pub use issue_license::handler as issue_license_handler;
pub use revoke_license::handler as revoke_license_handler;
pub use assert_license::handler as assert_license_handler;
