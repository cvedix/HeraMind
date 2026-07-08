//! Webhook push target (HTTP POST/PUT).

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::PushDestination;

/// Webhook configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Optional Bearer token.
    pub auth_token: Option<String>,
    /// Optional Basic auth (username:password).
    pub auth_basic: Option<BasicAuth>,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

fn default_method() -> String {
    "POST".to_string()
}

fn default_timeout() -> u64 {
    30
}

/// Webhook push destination.
pub struct WebhookTarget {
    client: reqwest::Client,
    config: WebhookConfig,
}

impl WebhookTarget {
    pub fn from_config(config: &serde_json::Value) -> Result<Self> {
        let wc: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| anyhow!("Invalid webhook config: {}", e))?;
        if wc.url.is_empty() {
            return Err(anyhow!("Webhook URL is required"));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(wc.timeout_secs))
            .build()?;
        Ok(Self { client, config: wc })
    }
}

#[async_trait]
impl PushDestination for WebhookTarget {
    async fn send(&self, payload: &str) -> Result<()> {
        let mut builder = match self.config.method.to_uppercase().as_str() {
            "PUT" => self.client.put(&self.config.url),
            _ => self.client.post(&self.config.url),
        };

        // Apply custom headers
        for (key, value) in &self.config.headers {
            builder = builder.header(key.as_str(), value.as_str());
        }

        // Apply auth
        if let Some(ref token) = self.config.auth_token {
            builder = builder.bearer_auth(token);
        } else if let Some(ref basic) = self.config.auth_basic {
            builder = builder.basic_auth(&basic.username, Some(&basic.password));
        }

        let response = builder
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Webhook returned status {}: {}",
                status,
                body.chars().take(500).collect::<String>()
            ));
        }

        Ok(())
    }

    fn validate_config(&self, config: &serde_json::Value) -> Result<()> {
        let wc: WebhookConfig = serde_json::from_value(config.clone())
            .map_err(|e| anyhow!("Invalid webhook config: {}", e))?;
        if wc.url.is_empty() {
            return Err(anyhow!("Webhook URL is required"));
        }
        Ok(())
    }
}
