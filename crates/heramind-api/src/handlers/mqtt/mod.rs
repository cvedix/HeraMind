//! MQTT management handlers.
//! Settings are now managed via the plugin system.

pub mod broker_config;
pub mod brokers;
pub mod cert_gen;
pub mod models;
pub mod status;
pub mod subscriptions;

// Re-export all handlers (excluding settings which are now plugin-managed)
pub use broker_config::*;
pub use brokers::*;
pub use status::*;
pub use subscriptions::*;
