//! Push target trait and registry.

pub mod mqtt;
pub mod webhook;

use crate::types::PushTargetType;
use anyhow::Result;
use async_trait::async_trait;

/// Trait for push destination implementations.
#[async_trait]
pub trait PushDestination: Send + Sync {
    /// Send a payload to the destination.
    async fn send(&self, payload: &str) -> Result<()>;

    /// Validate the configuration for this destination.
    fn validate_config(&self, config: &serde_json::Value) -> Result<()>;
}

/// Create a push destination from target type and config.
pub fn create_destination(
    target_type: &PushTargetType,
    config: &serde_json::Value,
) -> Result<Box<dyn PushDestination>> {
    let dest: Box<dyn PushDestination> = match target_type {
        PushTargetType::Webhook => {
            let target = webhook::WebhookTarget::from_config(config)?;
            Box::new(target)
        }
        PushTargetType::Mqtt => {
            let target = mqtt::MqttTarget::from_config(config)?;
            Box::new(target)
        }
    };
    Ok(dest)
}
