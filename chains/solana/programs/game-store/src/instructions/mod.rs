pub mod initialize;
pub mod set_price;
pub mod buy_game;
pub mod withdraw;
pub mod set_affiliate;
pub mod set_subscription;

pub use initialize::handler as initialize_handler;
pub use set_price::handler as set_price_handler;
pub use buy_game::handler as buy_game_handler;
pub use withdraw::handler as withdraw_handler;
pub use set_affiliate::handler as set_affiliate_handler;
pub use set_subscription::handler as set_subscription_handler;
