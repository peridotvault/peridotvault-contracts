pub mod initialize;
pub mod register_game;
pub mod update_game;
pub mod set_status;
pub mod transfer_publisher;

pub use initialize::handler as initialize_handler;
pub use register_game::handler as register_game_handler;
pub use update_game::handler as update_game_handler;
pub use set_status::handler as set_status_handler;
pub use transfer_publisher::handler as transfer_publisher_handler;
