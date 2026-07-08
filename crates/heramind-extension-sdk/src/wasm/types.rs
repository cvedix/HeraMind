//! WASM-specific type definitions
//!
//! This module provides types for WASM extensions that don't have
//! access to the full heramind-core crate.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// Re-export extension types
pub use crate::extension::*;

/// Host API for WASM extensions
pub struct Host;

impl Host {
    /// Make an HTTP request
    pub fn http_request(method: &str, url: &str) -> Result<serde_json::Value, String> {
        super::bindings::http_request(method, url)
    }

    /// Log a message
    pub fn log(level: &str, message: &str) {
        super::bindings::log(level, message)
    }

    /// Store a metric value
    pub fn store_metric(name: &str, value: &serde_json::Value) {
        super::bindings::store_metric(name, value)
    }

    /// Read a device metric
    pub fn device_read(device_id: &str, metric: &str) -> Result<serde_json::Value, String> {
        super::bindings::device_read(device_id, metric)
    }

    /// Write to a device
    pub fn device_write(
        device_id: &str,
        command: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        super::bindings::device_write(device_id, command, params)
    }
}

/// Thread-local metric storage for WASM extensions.
///
/// Using thread-local storage instead of `static mut` is safe because:
/// 1. Each WASM instance runs in its own thread
/// 2. No data races between threads
/// 3. Memory is automatically cleaned up when the thread exits
thread_local! {
    static WASM_METRIC_CACHE: RefCell<std::collections::HashMap<String, SdkMetricValue>> =
        RefCell::new(std::collections::HashMap::new());
}

/// Store a metric value in WASM cache
pub fn store_metric_value<T: Into<SdkMetricValue>>(name: &str, value: T) {
    let metric_value = value.into();
    WASM_METRIC_CACHE.with(|cache| {
        cache.borrow_mut().insert(name.to_string(), metric_value);
    });
}

/// Initialize the metric cache (now a no-op, kept for API compatibility)
pub fn init_metric_cache() {
    // Thread-local storage is automatically initialized
    // This function is kept for backward compatibility
}

/// Clear the metric cache
pub fn clear_metric_cache() {
    WASM_METRIC_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

/// Get all cached metrics as JSON
pub fn get_cached_metrics_json() -> String {
    WASM_METRIC_CACHE.with(|cache| {
        let cache = cache.borrow();
        let metrics: Vec<_> = cache
            .iter()
            .map(|(k, v)| {
                serde_json::json!({
                    "name": k,
                    "value": v,
                    "timestamp": 0
                })
            })
            .collect();
        serde_json::to_string(&metrics).unwrap_or_else(|_| "[]".to_string())
    })
}

/// Get the number of cached metrics (for testing/debugging)
pub fn cached_metrics_count() -> usize {
    WASM_METRIC_CACHE.with(|cache| cache.borrow().len())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_store_metric_value_string() {
        clear_metric_cache();
        store_metric_value("test_string", SdkMetricValue::String("hello".to_string()));

        assert_eq!(cached_metrics_count(), 1);
    }

    #[test]
    fn test_store_metric_value_float() {
        clear_metric_cache();
        store_metric_value("test_float", SdkMetricValue::Float(42.5));

        assert_eq!(cached_metrics_count(), 1);
    }

    #[test]
    fn test_store_metric_value_int() {
        clear_metric_cache();
        store_metric_value("test_int", SdkMetricValue::Int(100));

        assert_eq!(cached_metrics_count(), 1);
    }

    #[test]
    fn test_store_metric_value_bool() {
        clear_metric_cache();
        store_metric_value("test_bool", SdkMetricValue::Bool(true));

        assert_eq!(cached_metrics_count(), 1);
    }

    #[test]
    fn test_clear_metric_cache() {
        clear_metric_cache();
        store_metric_value("metric1", SdkMetricValue::Int(1));
        store_metric_value("metric2", SdkMetricValue::Int(2));
        assert_eq!(cached_metrics_count(), 2);

        clear_metric_cache();
        assert_eq!(cached_metrics_count(), 0);
    }

    #[test]
    fn test_get_cached_metrics_json_empty() {
        clear_metric_cache();
        let json = get_cached_metrics_json();

        assert_eq!(json, "[]");
    }

    #[test]
    fn test_get_cached_metrics_json_with_data() {
        clear_metric_cache();
        store_metric_value("temperature", SdkMetricValue::Float(25.5));
        store_metric_value("humidity", SdkMetricValue::Float(65.0));

        let json = get_cached_metrics_json();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.len(), 2);

        // Find temperature metric
        let temp_metric = parsed.iter().find(|m| m["name"] == "temperature").unwrap();
        assert_eq!(temp_metric["value"]["Float"], 25.5);

        // Find humidity metric
        let humidity_metric = parsed.iter().find(|m| m["name"] == "humidity").unwrap();
        assert_eq!(humidity_metric["value"]["Float"], 65.0);
    }

    #[test]
    fn test_metric_cache_overwrite() {
        clear_metric_cache();
        store_metric_value("metric", SdkMetricValue::Int(10));
        assert_eq!(cached_metrics_count(), 1);

        // Overwrite with new value
        store_metric_value("metric", SdkMetricValue::Int(20));
        assert_eq!(cached_metrics_count(), 1); // Count should still be 1

        let json = get_cached_metrics_json();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed[0]["value"]["Int"], 20);
    }

    #[test]
    fn test_init_metric_cache_noop() {
        // Just verify it doesn't panic
        init_metric_cache();
        init_metric_cache();
    }

    #[test]
    fn test_sdk_metric_value_serialization() {
        let float_val = SdkMetricValue::Float(3.14);
        let json = serde_json::to_string(&float_val).unwrap();
        assert!(json.contains("Float"));
        assert!(json.contains("3.14"));

        let int_val = SdkMetricValue::Int(42);
        let json = serde_json::to_string(&int_val).unwrap();
        assert!(json.contains("Int"));
        assert!(json.contains("42"));

        let string_val = SdkMetricValue::String("test".to_string());
        let json = serde_json::to_string(&string_val).unwrap();
        assert!(json.contains("String"));
        assert!(json.contains("test"));

        let bool_val = SdkMetricValue::Bool(true);
        let json = serde_json::to_string(&bool_val).unwrap();
        assert!(json.contains("Bool"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_sdk_metric_value_deserialization() {
        let json = r#"{"Float":2.718}"#;
        let value: SdkMetricValue = serde_json::from_str(json).unwrap();
        assert!(matches!(value, SdkMetricValue::Float(v) if (v - 2.718).abs() < 0.001));

        let json = r#"{"Int":100}"#;
        let value: SdkMetricValue = serde_json::from_str(json).unwrap();
        assert!(matches!(value, SdkMetricValue::Int(100)));

        let json = r#"{"String":"hello"}"#;
        let value: SdkMetricValue = serde_json::from_str(json).unwrap();
        assert!(matches!(value, SdkMetricValue::String(s) if s == "hello"));

        let json = r#"{"Bool":false}"#;
        let value: SdkMetricValue = serde_json::from_str(json).unwrap();
        assert!(matches!(value, SdkMetricValue::Bool(false)));
    }

    #[test]
    fn test_host_static_methods() {
        // Test that Host methods exist and compile
        // (They will fail at runtime without actual host implementation)
        let _ = Host::http_request("GET", "http://example.com");
        Host::log("info", "test message");
        let _ = Host::device_read("device-1", "temperature");
        let _ = Host::device_write("device-1", "set_level", &json!({"level": 80}));
        Host::store_metric("virtual_metric", &json!(42));
    }

    #[test]
    fn test_multiple_metrics_storage() {
        clear_metric_cache();

        // Store multiple metrics of different types
        store_metric_value("counter", SdkMetricValue::Int(100));
        store_metric_value("gauge", SdkMetricValue::Float(3.14159));
        store_metric_value("label", SdkMetricValue::String("active".to_string()));
        store_metric_value("flag", SdkMetricValue::Bool(true));

        assert_eq!(cached_metrics_count(), 4);

        let json = get_cached_metrics_json();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.len(), 4);
    }

    #[test]
    fn test_metric_value_into_conversion() {
        // Test that Into trait is implemented
        let value: SdkMetricValue = 42i64.into();
        assert!(matches!(value, SdkMetricValue::Int(42)));

        let value: SdkMetricValue = 3.14f64.into();
        assert!(matches!(value, SdkMetricValue::Float(v) if (v - 3.14).abs() < 0.001));

        let value: SdkMetricValue = true.into();
        assert!(matches!(value, SdkMetricValue::Bool(true)));

        let value: SdkMetricValue = "test".to_string().into();
        assert!(matches!(value, SdkMetricValue::String(s) if s == "test"));
    }
}
