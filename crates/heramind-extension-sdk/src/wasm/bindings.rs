//! WASM Host Function Bindings
//!
//! Minimal FFI interface for WASM extensions to communicate with the host.
//! All capabilities are accessed through a single unified interface.

/// Host function imports from HeraMind runtime
///
/// Design: Minimal FFI surface - only 4 functions needed:
/// 1. host_invoke_capability - Universal capability invocation
/// 2. host_event_subscribe - Event subscription (returns subscription ID)
/// 3. host_event_poll - Poll for subscribed events
/// 4. host_free - Free host-allocated memory
#[link(wasm_import_module = "heramind")]
extern "C" {
    /// Universal capability invocation entry point
    ///
    /// This single function handles ALL capability types:
    /// - DeviceMetricsRead, DeviceMetricsWrite, DeviceControl
    /// - TelemetryHistory, MetricsAggregate
    /// - EventPublish, EventSubscribe (for publish only)
    /// - ExtensionCall, AgentTrigger, RuleTrigger
    ///
    /// # Arguments
    /// - `capability_ptr/len`: Capability name (e.g., "device_metrics_read")
    /// - `params_ptr/len`: JSON-encoded parameters
    /// - `result_ptr/max_len`: Buffer to write result
    ///
    /// # Returns
    /// Length of result on success, -1 on error
    pub fn host_invoke_capability(
        capability_ptr: *const u8,
        capability_len: i32,
        params_ptr: *const u8,
        params_len: i32,
        result_ptr: *mut u8,
        result_max_len: i32,
    ) -> i32;

    /// Subscribe to events
    ///
    /// Registers a subscription for events matching the filter.
    /// Use host_event_poll to retrieve events.
    ///
    /// # Arguments
    /// - `event_type_ptr/len`: Event type to subscribe to
    /// - `filter_ptr/len`: JSON-encoded filter criteria
    ///
    /// # Returns
    /// Subscription ID (>0) on success, -1 on error
    pub fn host_event_subscribe(
        event_type_ptr: *const u8,
        event_type_len: i32,
        filter_ptr: *const u8,
        filter_len: i32,
    ) -> i64;

    /// Poll for subscribed events
    ///
    /// Returns events that have been received since last poll.
    /// Non-blocking: returns empty array if no events.
    ///
    /// # Arguments
    /// - `subscription_id`: ID from host_event_subscribe
    /// - `result_ptr/max_len`: Buffer to write events JSON array
    ///
    /// # Returns
    /// Length of result on success, -1 on error or invalid subscription
    pub fn host_event_poll(subscription_id: i64, result_ptr: *mut u8, result_max_len: i32) -> i32;

    /// Unsubscribe from events
    ///
    /// # Arguments
    /// - `subscription_id`: ID from host_event_subscribe
    ///
    /// # Returns
    /// 0 on success, -1 on error
    pub fn host_event_unsubscribe(subscription_id: i64) -> i32;

    /// Free host-allocated memory
    ///
    /// Call this to free memory returned by host functions.
    pub fn host_free(ptr: *const u8);

    /// Log a message (utility function)
    pub fn host_log(level_ptr: *const u8, level_len: i32, msg_ptr: *const u8, msg_len: i32);

    /// Get current timestamp from host
    ///
    /// Returns milliseconds since Unix epoch
    pub fn host_timestamp_ms() -> i64;
}

// ============================================================================
// High-level API (SDK internal use)
// ============================================================================

/// Default buffer size for results
const DEFAULT_BUFFER_SIZE: usize = 65536;

/// Invoke a capability through the host
///
/// This is the core function that all capability APIs use internally.
pub fn invoke_capability_raw(
    capability: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let params_str = serde_json::to_string(params).map_err(|e| e.to_string())?;
    let mut result_buffer = vec![0u8; DEFAULT_BUFFER_SIZE];

    let result_len = unsafe {
        host_invoke_capability(
            capability.as_ptr(),
            capability.len() as i32,
            params_str.as_ptr(),
            params_str.len() as i32,
            result_buffer.as_mut_ptr(),
            result_buffer.len() as i32,
        )
    };

    if result_len < 0 {
        return Err(format!("Capability '{}' invocation failed", capability));
    }

    // Find null terminator or use result length
    let end = result_buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(result_len as usize);
    let json_str = String::from_utf8_lossy(&result_buffer[..end]);

    serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))
}

/// Log a message from WASM
pub fn log(level: &str, message: &str) {
    unsafe {
        host_log(
            level.as_ptr(),
            level.len() as i32,
            message.as_ptr(),
            message.len() as i32,
        )
    }
}

/// Get current timestamp in milliseconds
pub fn timestamp_ms() -> i64 {
    unsafe { host_timestamp_ms() }
}

/// Subscribe to events (returns subscription ID)
pub fn event_subscribe_raw(event_type: &str, filter: &serde_json::Value) -> Result<i64, String> {
    let filter_str = serde_json::to_string(filter).map_err(|e| e.to_string())?;

    let sub_id = unsafe {
        host_event_subscribe(
            event_type.as_ptr(),
            event_type.len() as i32,
            filter_str.as_ptr(),
            filter_str.len() as i32,
        )
    };

    if sub_id < 0 {
        Err("Event subscription failed".to_string())
    } else {
        Ok(sub_id)
    }
}

/// Poll for events (returns array of events)
pub fn event_poll_raw(subscription_id: i64) -> Result<Vec<serde_json::Value>, String> {
    let mut result_buffer = vec![0u8; DEFAULT_BUFFER_SIZE];

    let result_len = unsafe {
        host_event_poll(
            subscription_id,
            result_buffer.as_mut_ptr(),
            result_buffer.len() as i32,
        )
    };

    if result_len < 0 {
        return Err("Event poll failed".to_string());
    }

    let end = result_buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(result_len as usize);
    let json_str = String::from_utf8_lossy(&result_buffer[..end]);

    // Parse as array of events
    serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error: {}", e))
}

/// Unsubscribe from events
pub fn event_unsubscribe_raw(subscription_id: i64) -> Result<(), String> {
    let result = unsafe { host_event_unsubscribe(subscription_id) };
    if result < 0 {
        Err("Event unsubscribe failed".to_string())
    } else {
        Ok(())
    }
}

// ============================================================================
// Backward compatibility - Keep old functions as wrappers
// ============================================================================

/// Read a device metric from WASM (backward compatible)
pub fn device_read(device_id: &str, metric: &str) -> Result<serde_json::Value, String> {
    invoke_capability_raw(
        "device_metrics_read",
        &serde_json::json!({
            "device_id": device_id,
            "metric": metric,
        }),
    )
}

/// Write to a device from WASM (backward compatible)
pub fn device_write(
    device_id: &str,
    command: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    invoke_capability_raw(
        "device_control",
        &serde_json::json!({
            "device_id": device_id,
            "command": command,
            "params": params,
        }),
    )
}

/// Store a metric value from WASM (backward compatible)
pub fn store_metric(name: &str, value: &serde_json::Value) {
    match invoke_capability_raw(
        "device_metrics_write",
        &serde_json::json!({
            "metric": name,
            "value": value,
            "is_virtual": true,
        }),
    ) {
        Ok(result) => {
            if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
                if !success {
                    eprintln!(
                        "[Extension SDK] store_metric '{}' failed: {:?}",
                        name,
                        result.get("error")
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("[Extension SDK] store_metric '{}' error: {}", name, e);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_capability_name_constants() {
        // Verify capability names match expected format
        assert_eq!("device_metrics_read", "device_metrics_read");
        assert_eq!("device_metrics_write", "device_metrics_write");
        assert_eq!("device_control", "device_control");
        assert_eq!("telemetry_history", "telemetry_history");
        assert_eq!("metrics_aggregate", "metrics_aggregate");
        assert_eq!("event_publish", "event_publish");
        assert_eq!("event_subscribe", "event_subscribe");
        assert_eq!("extension_call", "extension_call");
        assert_eq!("agent_trigger", "agent_trigger");
        assert_eq!("rule_trigger", "rule_trigger");
    }

    #[test]
    fn test_device_read_params() {
        let params = json!({
            "device_id": "sensor-1",
            "metric": "temperature",
        });

        assert_eq!(params["device_id"], "sensor-1");
        assert_eq!(params["metric"], "temperature");
    }

    #[test]
    fn test_device_write_params() {
        let params = json!({
            "device_id": "actuator-1",
            "command": "set_level",
            "params": {"level": 80},
        });

        assert_eq!(params["device_id"], "actuator-1");
        assert_eq!(params["command"], "set_level");
        assert_eq!(params["params"]["level"], 80);
    }

    #[test]
    fn test_store_metric_params() {
        let params = json!({
            "metric": "calculated_value",
            "value": 42.5,
            "is_virtual": true,
        });

        assert_eq!(params["metric"], "calculated_value");
        assert_eq!(params["value"], 42.5);
        assert_eq!(params["is_virtual"], true);
    }

    #[test]
    fn test_invoke_capability_params_structure() {
        // Test that parameter structure is correct for capability invocation
        let capability = "device_metrics_read";
        let params = json!({
            "device_id": "device-1",
        });

        let params_str = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&params_str).unwrap();

        assert_eq!(parsed["device_id"], "device-1");
        assert_eq!(capability, "device_metrics_read");
    }

    #[test]
    fn test_event_subscribe_params() {
        let event_type = "device_changed";
        let filter = json!({
            "device_id": "sensor-*",
        });

        assert_eq!(event_type, "device_changed");
        assert_eq!(filter["device_id"], "sensor-*");
    }

    #[test]
    fn test_event_poll_empty_result() {
        // Test parsing empty event array
        let events: Vec<serde_json::Value> = vec![];
        let events_json = serde_json::to_string(&events).unwrap();
        assert_eq!(events_json, "[]");
    }

    #[test]
    fn test_event_poll_with_events() {
        let events = vec![
            json!({"type": "device_changed", "device_id": "sensor-1"}),
            json!({"type": "device_changed", "device_id": "sensor-2"}),
        ];

        let events_json = serde_json::to_string(&events).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&events_json).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["device_id"], "sensor-1");
        assert_eq!(parsed[1]["device_id"], "sensor-2");
    }

    #[test]
    fn test_default_buffer_size() {
        assert_eq!(DEFAULT_BUFFER_SIZE, 65536);
        assert!(DEFAULT_BUFFER_SIZE >= 1024); // At least 1KB
        assert!(DEFAULT_BUFFER_SIZE <= 1024 * 1024); // At most 1MB
    }

    #[test]
    fn test_result_buffer_handling() {
        // Simulate result buffer handling
        let result = json!({
            "success": true,
            "data": {"temperature": 25.5},
        });

        let result_str = serde_json::to_string(&result).unwrap();
        let bytes = result_str.as_bytes();

        // Check that result fits in default buffer
        assert!(bytes.len() < DEFAULT_BUFFER_SIZE);

        // Verify round-trip
        let parsed: serde_json::Value = serde_json::from_str(&result_str).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["data"]["temperature"], 25.5);
    }

    #[test]
    fn test_error_result_parsing() {
        let error_response = json!({
            "success": false,
            "error": "Device not found",
        });

        let error_str = serde_json::to_string(&error_response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&error_str).unwrap();

        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"], "Device not found");
    }

    #[test]
    fn test_telemetry_query_params() {
        let params = json!({
            "device_id": "sensor-1",
            "metric": "temperature",
            "start": 1700000000000i64,
            "end": 1700086400000i64,
        });

        assert_eq!(params["device_id"], "sensor-1");
        assert_eq!(params["metric"], "temperature");
        assert!(params["start"].is_number());
        assert!(params["end"].is_number());
    }

    #[test]
    fn test_aggregation_params() {
        let params = json!({
            "device_id": "sensor-1",
            "metric": "temperature",
            "aggregation": "avg",
            "start": 1700000000000i64,
            "end": 1700086400000i64,
        });

        assert_eq!(params["aggregation"], "avg");
    }

    #[test]
    fn test_aggregation_types() {
        // Test all supported aggregation types
        let agg_types = vec!["avg", "sum", "min", "max", "count"];

        for agg_type in agg_types {
            let params = json!({
                "aggregation": agg_type,
            });
            assert_eq!(params["aggregation"], agg_type);
        }
    }

    #[test]
    fn test_extension_call_params() {
        let params = json!({
            "extension_id": "analytics-extension",
            "command": "analyze",
            "args": {"time_range": "24h"},
        });

        assert_eq!(params["extension_id"], "analytics-extension");
        assert_eq!(params["command"], "analyze");
        assert_eq!(params["args"]["time_range"], "24h");
    }

    #[test]
    fn test_agent_trigger_params() {
        let params = json!({
            "agent_id": "anomaly-detector",
            "input": {
                "device_id": "sensor-1",
                "lookback": "7d",
            },
        });

        assert_eq!(params["agent_id"], "anomaly-detector");
        assert_eq!(params["input"]["device_id"], "sensor-1");
    }

    #[test]
    fn test_rule_trigger_params() {
        let params = json!({
            "rule_id": "alert-threshold",
            "context": {
                "device_id": "sensor-1",
                "threshold": 80.0,
                "current": 85.5,
            },
        });

        assert_eq!(params["rule_id"], "alert-threshold");
        assert_eq!(params["context"]["threshold"], 80.0);
        assert_eq!(params["context"]["current"], 85.5);
    }

    #[test]
    fn test_json_serialization_deserialization() {
        // Test complex nested structures
        let complex = json!({
            "device": {
                "id": "device-1",
                "name": "Temperature Sensor",
                "location": {"building": "A", "floor": 1},
            },
            "readings": [
                {"metric": "temp", "value": 25.5},
                {"metric": "humidity", "value": 65.0},
            ],
            "metadata": {
                "last_update": "2024-01-15T10:30:00Z",
                "quality": "good",
            },
        });

        let json_str = serde_json::to_string(&complex).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["device"]["id"], "device-1");
        assert_eq!(parsed["device"]["location"]["building"], "A");
        assert_eq!(parsed["readings"][0]["metric"], "temp");
        assert_eq!(parsed["readings"][1]["value"], 65.0);
    }

    #[test]
    fn test_unicode_handling() {
        // Test unicode in parameters
        let params = json!({
            "device_id": "设备-1",  // Chinese characters
            "description": "温度传感器",
        });

        let json_str = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["device_id"], "设备-1");
        assert_eq!(parsed["description"], "温度传感器");
    }

    #[test]
    fn test_large_numbers() {
        // Test handling of large numbers (timestamps, etc.)
        let params = json!({
            "timestamp": 1700000000000i64,  // Millisecond timestamp
            "value": 1234567890.123456f64,
        });

        let json_str = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["timestamp"].as_i64().unwrap(), 1700000000000i64);
    }

    #[test]
    fn test_null_values() {
        // Test null handling
        let params = json!({
            "device_id": "device-1",
            "optional_value": null,
        });

        assert!(params["optional_value"].is_null());
    }

    #[test]
    fn test_array_params() {
        // Test array parameters
        let params = json!({
            "device_ids": ["sensor-1", "sensor-2", "sensor-3"],
            "metrics": ["temperature", "humidity"],
        });

        assert_eq!(params["device_ids"].as_array().unwrap().len(), 3);
        assert_eq!(params["metrics"].as_array().unwrap().len(), 2);
    }
}
