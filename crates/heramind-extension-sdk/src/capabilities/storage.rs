//! Storage Capabilities (Unified for Native and WASM)
//!
//! This module provides storage-related capabilities with a unified API
//! that works on both Native and WASM targets.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[cfg(not(target_arch = "wasm32"))]
use crate::host::*;

#[cfg(target_arch = "wasm32")]
use crate::wasm::{capabilities, ExtensionContext};

/// Capability error type
pub type CapabilityError = String;

#[cfg(not(target_arch = "wasm32"))]
pub type Context = ExtensionContext;

#[cfg(target_arch = "wasm32")]
pub type Context = crate::wasm::ExtensionContext;

// ============================================================================
// Query Types
// ============================================================================

/// Query type for storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    /// Query latest value(s)
    Latest,
    /// Query range of values
    Range { start: i64, end: i64 },
}

// ============================================================================
// Latest Query
// ============================================================================

/// Query the latest value for a specific metric
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_latest(
    context: &Context,
    device_id: &str,
    metric: &str,
) -> Result<Option<MetricValue>, CapabilityError> {
    let result = context
        .invoke_capability(
            ExtensionCapability::StorageQuery,
            &json!({
                "query": "latest",
                "params": {
                    "device_id": device_id,
                    "metric": metric,
                }
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    if result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        Ok(Some(MetricValue {
            value: result.get("value").cloned().unwrap_or(json!(null)),
            timestamp: result
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            quality: result.get("quality").and_then(|v| v.as_f64()),
        }))
    } else {
        Ok(None)
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_latest(
    context: &Context,
    device_id: &str,
    metric: &str,
) -> Result<Option<MetricValue>, CapabilityError> {
    let result = context.invoke_capability(
        capabilities::STORAGE_QUERY,
        &json!({
            "query": "latest",
            "params": {
                "device_id": device_id,
                "metric": metric,
            }
        }),
    )?;

    if result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        Ok(Some(MetricValue {
            value: result.get("value").cloned().unwrap_or(json!(null)),
            timestamp: result
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            quality: result.get("quality").and_then(|v| v.as_f64()),
        }))
    } else {
        Ok(None)
    }
}

// ============================================================================
// All Metrics Query
// ============================================================================

/// Query all latest metrics for a device
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_all_latest(
    context: &Context,
    device_id: &str,
) -> Result<DeviceMetrics, CapabilityError> {
    let result = context
        .invoke_capability(
            ExtensionCapability::StorageQuery,
            &json!({
                "query": "latest",
                "params": {
                    "device_id": device_id,
                }
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    let metrics = result
        .get("metrics")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    let value = MetricValue {
                        value: v.get("value").cloned().unwrap_or(json!(null)),
                        timestamp: v.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
                        quality: v.get("quality").and_then(|q| q.as_f64()),
                    };
                    (k.clone(), value)
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(DeviceMetrics {
        device_id: device_id.to_string(),
        metrics,
    })
}

#[cfg(target_arch = "wasm32")]
pub fn get_all_latest(
    context: &Context,
    device_id: &str,
) -> Result<DeviceMetrics, CapabilityError> {
    let result = context.invoke_capability(
        capabilities::STORAGE_QUERY,
        &json!({
            "query": "latest",
            "params": {
                "device_id": device_id,
            }
        }),
    )?;

    let metrics = result
        .get("metrics")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    let value = MetricValue {
                        value: v.get("value").cloned().unwrap_or(json!(null)),
                        timestamp: v.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
                        quality: v.get("quality").and_then(|q| q.as_f64()),
                    };
                    (k.clone(), value)
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(DeviceMetrics {
        device_id: device_id.to_string(),
        metrics,
    })
}

// ============================================================================
// Range Query
// ============================================================================

/// Query a range of values for a metric
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_range(
    context: &Context,
    device_id: &str,
    metric: &str,
    start: i64,
    end: i64,
) -> Result<Vec<MetricValue>, CapabilityError> {
    let result = context
        .invoke_capability(
            ExtensionCapability::StorageQuery,
            &json!({
                "query": "range",
                "params": {
                    "device_id": device_id,
                    "metric": metric,
                    "start": start,
                    "end": end,
                }
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    let data = result
        .get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|d| MetricValue {
                    value: d.get("value").cloned().unwrap_or(json!(null)),
                    timestamp: d.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
                    quality: d.get("quality").and_then(|q| q.as_f64()),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(data)
}

#[cfg(target_arch = "wasm32")]
pub fn get_range(
    context: &Context,
    device_id: &str,
    metric: &str,
    start: i64,
    end: i64,
) -> Result<Vec<MetricValue>, CapabilityError> {
    let result = context.invoke_capability(
        capabilities::STORAGE_QUERY,
        &json!({
            "query": "range",
            "params": {
                "device_id": device_id,
                "metric": metric,
                "start": start,
                "end": end,
            }
        }),
    )?;

    let data = result
        .get("data")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|d| MetricValue {
                    value: d.get("value").cloned().unwrap_or(json!(null)),
                    timestamp: d.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
                    quality: d.get("quality").and_then(|q| q.as_f64()),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(data)
}

// ============================================================================
// Data Types
// ============================================================================

/// A single metric value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// The metric value
    pub value: Value,
    /// Timestamp in milliseconds
    pub timestamp: i64,
    /// Quality indicator (0.0 - 1.0)
    pub quality: Option<f64>,
}

/// All metrics for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceMetrics {
    /// Device ID
    pub device_id: String,
    /// Metric name -> value mapping
    pub metrics: std::collections::HashMap<String, MetricValue>,
}

impl MetricValue {
    /// Get value as f64
    pub fn as_f64(&self) -> Option<f64> {
        self.value.as_f64()
    }

    /// Get value as i64
    pub fn as_i64(&self) -> Option<i64> {
        self.value.as_i64()
    }

    /// Get value as string
    pub fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }

    /// Get value as boolean
    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_conversions() {
        let mv = MetricValue {
            value: json!(42.5),
            timestamp: 1000,
            quality: Some(0.95),
        };

        assert_eq!(mv.as_f64(), Some(42.5));
        assert_eq!(mv.timestamp, 1000);
        assert_eq!(mv.quality, Some(0.95));
    }

    #[test]
    fn test_metric_value_string() {
        let mv = MetricValue {
            value: json!("hello"),
            timestamp: 2000,
            quality: None,
        };

        assert_eq!(mv.as_str(), Some("hello"));
        assert_eq!(mv.as_f64(), None);
    }

    #[test]
    fn test_device_metrics() {
        let mut metrics = std::collections::HashMap::new();
        metrics.insert(
            "temperature".to_string(),
            MetricValue {
                value: json!(25.5),
                timestamp: 1000,
                quality: Some(1.0),
            },
        );

        let dm = DeviceMetrics {
            device_id: "sensor-001".to_string(),
            metrics,
        };

        assert_eq!(dm.device_id, "sensor-001");
        assert!(dm.metrics.contains_key("temperature"));
    }
}
