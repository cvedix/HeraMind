//! Device management handlers.
//!
//! Provides REST API for device management with MDL support.

pub mod auto_onboard;
pub mod ble_provision;
pub mod compat;
pub mod crud;
pub mod mdl;
pub mod metrics;
pub mod models;
pub mod telemetry;
pub mod telemetry_stats;
pub mod types;
pub mod webhook;

// Re-export all handlers for use in routing
pub use auto_onboard::*;
pub use ble_provision::*;
pub use crud::*;
pub use mdl::*;
pub use metrics::*;
pub use telemetry::*;
pub use telemetry_stats::*;
pub use types::*;
pub use webhook::*;
