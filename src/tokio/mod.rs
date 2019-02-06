//! Async implementation based on tokio and futures

mod gateway;
mod search;

pub use self::gateway::Gateway;
pub(crate) use self::search::get_control_url;
pub use self::search::{search_gateway, search_gateway_from, search_gateway_from_timeout, search_gateway_timeout};
