//! Device integration for rule engine.
//!
//! Provides device command execution with retry logic.

use std::collections::HashMap;
use std::sync::Arc;

use heramind_devices::{DeviceService, MetricValue as DeviceMetricValue};
use serde::{Deserialize, Serialize};

/// Retry configuration for device command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
        }
    }
}

impl RetryConfig {
    pub fn delay_for_attempt(&self, attempt: u32) -> std::time::Duration {
        if attempt == 0 {
            return std::time::Duration::from_millis(0);
        }
        let delay_ms = (self.initial_delay_ms as f64
            * self.backoff_multiplier.powi(attempt as i32 - 1))
        .min(self.max_delay_ms as f64) as u64;
        std::time::Duration::from_millis(delay_ms)
    }
}

/// Device action executor for rule engine.
///
/// Executes commands on devices via `DeviceService` with retry logic.
pub struct DeviceActionExecutor {
    device_service: Option<Arc<DeviceService>>,
    retry_config: RetryConfig,
}

impl DeviceActionExecutor {
    pub fn with_device_service(device_service: Arc<DeviceService>) -> Self {
        Self {
            device_service: Some(device_service),
            retry_config: RetryConfig::default(),
        }
    }

    /// Execute a command with retry logic.
    pub async fn execute_command_with_retry(
        &self,
        device_id: &str,
        command: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<DeviceMetricValue>, String> {
        let max_attempts = self.retry_config.max_retries + 1;
        let mut last_error = String::new();

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = self.retry_config.delay_for_attempt(attempt);
                tracing::info!(
                    "Retrying command '{}' on device '{}' (attempt {}/{}) after {:?}",
                    command,
                    device_id,
                    attempt + 1,
                    max_attempts,
                    delay
                );
                tokio::time::sleep(delay).await;
            }

            if let Some(ref device_service) = self.device_service {
                match device_service
                    .send_command(device_id, command, params.clone())
                    .await
                {
                    Ok(result) => {
                        if attempt > 0 {
                            tracing::info!(
                                "Command '{}' on device '{}' succeeded on attempt {}",
                                command,
                                device_id,
                                attempt + 1
                            );
                        }
                        return Ok(result);
                    }
                    Err(e) => {
                        last_error = e.to_string();
                        if !self.is_error_retryable(&last_error) {
                            return Err(last_error);
                        }
                        if attempt < max_attempts - 1 {
                            tracing::warn!(
                                "Command '{}' on device '{}' failed (attempt {}): {}",
                                command,
                                device_id,
                                attempt + 1,
                                last_error
                            );
                        }
                    }
                }
            } else {
                return Err("No device service configured".to_string());
            }
        }

        tracing::error!(
            "Command '{}' on device '{}' failed after {} attempts: {}",
            command,
            device_id,
            max_attempts,
            last_error
        );
        Err(last_error)
    }

    fn is_error_retryable(&self, error: &str) -> bool {
        let e = error.to_lowercase();
        !(e.contains("not found")
            || e.contains("invalid parameter")
            || e.contains("permission denied")
            || e.contains("unauthorized"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_delay() {
        let config = RetryConfig::default();
        assert_eq!(
            config.delay_for_attempt(0),
            std::time::Duration::from_millis(0)
        );
        assert!(config.delay_for_attempt(1) > std::time::Duration::from_millis(0));
    }
}
