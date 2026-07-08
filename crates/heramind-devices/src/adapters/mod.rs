//! Device adapters for HeraMind event-driven architecture.
//!
//! This module contains implementations of the `DeviceAdapter` trait
//! for various data sources and protocols.
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `mqtt` | MQTT protocol support (default) |
//! | `webhook` | Webhook adapter (default) |
//! | `embedded-broker` | Embedded MQTT broker |

// MQTT adapter (feature-gated)
#[cfg(feature = "mqtt")]
pub mod mqtt;
#[cfg(feature = "mqtt")]
pub use mqtt::{create_mqtt_adapter, MqttAdapter, MqttAdapterConfig};

// Webhook adapter (always available)
pub mod webhook;
pub use webhook::{create_webhook_adapter, WebhookAdapter, WebhookAdapterConfig, WebhookPayload};

use crate::adapter::{AdapterResult, DeviceAdapter};
use heramind_core::EventBus;
use serde_json::Value;
use std::sync::Arc;

/// Create an adapter by type identifier.
///
/// This function provides a unified way to create device adapters
/// based on configuration, with feature-gated compilation.
pub fn create_adapter(
    adapter_type: &str,
    config: &Value,
    event_bus: &EventBus,
) -> AdapterResult<Arc<dyn DeviceAdapter>> {
    match adapter_type {
        #[cfg(feature = "mqtt")]
        "mqtt" => {
            let cfg: MqttAdapterConfig = serde_json::from_value(config.clone()).map_err(|e| {
                crate::adapter::AdapterError::Configuration(format!("Invalid MQTT config: {}", e))
            })?;
            let device_registry = Arc::new(crate::registry::DeviceRegistry::new());
            Ok(create_mqtt_adapter(cfg, event_bus, device_registry))
        }
        "webhook" => {
            let cfg: WebhookAdapterConfig =
                serde_json::from_value(config.clone()).map_err(|e| {
                    crate::adapter::AdapterError::Configuration(format!(
                        "Invalid Webhook config: {}",
                        e
                    ))
                })?;
            let device_registry = Arc::new(crate::registry::DeviceRegistry::new());
            Ok(create_webhook_adapter(cfg, event_bus, device_registry))
        }
        _ => Err(crate::adapter::AdapterError::Configuration(format!(
            "Unknown adapter type: {}. Available adapters: {}",
            adapter_type,
            available_adapters().join(", ")
        ))),
    }
}

/// Get list of available adapter types (based on enabled features).
#[allow(clippy::vec_init_then_push)]
pub fn available_adapters() -> Vec<&'static str> {
    let mut adapters = Vec::with_capacity(2);

    #[cfg(feature = "mqtt")]
    adapters.push("mqtt");

    adapters.push("webhook");

    adapters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_adapters() {
        let adapters = available_adapters();
        assert!(!adapters.is_empty());
        assert!(adapters.contains(&"webhook"));
    }

    #[test]
    fn test_create_adapter_unknown() {
        let event_bus = EventBus::new();
        let result = create_adapter("unknown", &serde_json::json!({}), &event_bus);
        assert!(result.is_err());
    }
}
