//! WASM Extension Context
//!
//! Lightweight context for WASM extensions that provides unified
//! capability invocation through the host interface.

use serde_json::Value;

/// Capability names (must match heramind-core::extension::context::ExtensionCapability)
pub mod capabilities {
    pub const DEVICE_METRICS_READ: &str = "device_metrics_read";
    pub const DEVICE_METRICS_WRITE: &str = "device_metrics_write";
    pub const DEVICE_CONTROL: &str = "device_control";
    pub const STORAGE_QUERY: &str = "storage_query";
    pub const EVENT_PUBLISH: &str = "event_publish";
    pub const EVENT_SUBSCRIBE: &str = "event_subscribe";
    pub const TELEMETRY_HISTORY: &str = "telemetry_history";
    pub const METRICS_AGGREGATE: &str = "metrics_aggregate";
    pub const EXTENSION_CALL: &str = "extension_call";
    pub const AGENT_TRIGGER: &str = "agent_trigger";
    pub const CHAT_STREAM: &str = "chat_stream";
    pub const CHAT_STREAM_CANCEL: &str = "chat_stream_cancel";
    pub const CHAT_SESSION_OPEN: &str = "chat_session_open";
    pub const CHAT_SESSION_SEND: &str = "chat_session_send";
    pub const CHAT_SESSION_CLOSE: &str = "chat_session_close";
    pub const CHAT_STREAM_CANCEL_TURN: &str = "chat_stream_cancel_turn";
    pub const RULE_TRIGGER: &str = "rule_trigger";
}

/// WASM Extension Context
///
/// Provides unified API for capability invocation that mirrors
/// the Native ExtensionContext API.
#[derive(Debug, Clone)]
pub struct ExtensionContext {
    /// Extension ID
    pub extension_id: String,
}

impl ExtensionContext {
    /// Create a new WASM extension context
    pub fn new(extension_id: String) -> Self {
        Self { extension_id }
    }

    /// Get extension ID
    pub fn extension_id(&self) -> &str {
        &self.extension_id
    }

    /// Invoke a capability through the host
    ///
    /// This is the core method that all capability APIs use.
    /// It provides a unified interface that works identically to Native.
    ///
    /// # Example
    /// ```ignore
    /// let result = ctx.invoke_capability("device_metrics_read", json!({"device_id": "sensor-1"}))?;
    /// ```
    pub fn invoke_capability(&self, capability: &str, params: &Value) -> Result<Value, String> {
        // Call host through bindings
        crate::wasm::bindings::invoke_capability_raw(capability, params)
    }

    // ========================================================================
    // Convenience methods for common capabilities
    // ========================================================================

    /// Read device metrics
    pub fn device_read(&self, device_id: &str, metric: &str) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::DEVICE_METRICS_READ,
            &serde_json::json!({
                "device_id": device_id,
                "metric": metric,
            }),
        )
    }

    /// Write device metric (virtual metric)
    pub fn device_write(&self, device_id: &str, key: &str, value: &Value) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::DEVICE_METRICS_WRITE,
            &serde_json::json!({
                "device_id": device_id,
                "key": key,
                "value": value,
                "is_virtual": true,
            }),
        )
    }

    /// Send device command
    pub fn device_command(
        &self,
        device_id: &str,
        command: &str,
        params: &Value,
    ) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::DEVICE_CONTROL,
            &serde_json::json!({
                "device_id": device_id,
                "command": command,
                "params": params,
            }),
        )
    }

    /// Query telemetry history
    pub fn query_telemetry(
        &self,
        device_id: &str,
        metric: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::TELEMETRY_HISTORY,
            &serde_json::json!({
                "device_id": device_id,
                "metric": metric,
                "start": start_time,
                "end": end_time,
            }),
        )
    }

    /// Aggregate metrics
    pub fn aggregate_metrics(
        &self,
        device_id: &str,
        metric: &str,
        aggregation: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::METRICS_AGGREGATE,
            &serde_json::json!({
                "device_id": device_id,
                "metric": metric,
                "aggregation": aggregation,
                "start": start_time,
                "end": end_time,
            }),
        )
    }

    /// Publish event
    pub fn publish_event(&self, event_type: &str, payload: &Value) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::EVENT_PUBLISH,
            &serde_json::json!({
                "event_type": event_type,
                "payload": payload,
            }),
        )
    }

    /// Call another extension
    pub fn call_extension(
        &self,
        extension_id: &str,
        command: &str,
        args: &Value,
    ) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::EXTENSION_CALL,
            &serde_json::json!({
                "extension_id": extension_id,
                "command": command,
                "args": args,
            }),
        )
    }

    /// Trigger agent
    pub fn trigger_agent(&self, agent_id: &str, input: &Value) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::AGENT_TRIGGER,
            &serde_json::json!({
                "agent_id": agent_id,
                "input": input,
            }),
        )
    }

    /// Trigger rule
    pub fn trigger_rule(&self, rule_id: &str, context: &Value) -> Result<Value, String> {
        self.invoke_capability(
            capabilities::RULE_TRIGGER,
            &serde_json::json!({
                "rule_id": rule_id,
                "context": context,
            }),
        )
    }
}

/// Event subscription handle
pub struct EventSubscription {
    pub id: i64,
    event_type: String,
}

impl EventSubscription {
    /// Create a new subscription handle
    pub fn new(id: i64, event_type: String) -> Self {
        Self { id, event_type }
    }

    /// Poll for new events
    ///
    /// Returns an array of events that have been received since last poll.
    /// Non-blocking: returns empty array if no events.
    pub fn poll(&self) -> Result<Vec<Value>, String> {
        crate::wasm::bindings::event_poll_raw(self.id)
    }

    /// Get the event type
    pub fn event_type(&self) -> &str {
        &self.event_type
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        let _ = crate::wasm::bindings::event_unsubscribe_raw(self.id);
    }
}

/// Event subscription helper
impl ExtensionContext {
    /// Subscribe to events
    ///
    /// Returns a subscription handle that can be used to poll for events.
    pub fn subscribe_event(
        &self,
        event_type: &str,
        filter: Option<&Value>,
    ) -> Result<EventSubscription, String> {
        let default_filter = serde_json::json!({});
        let filter = filter.unwrap_or(&default_filter);
        let id = crate::wasm::bindings::event_subscribe_raw(event_type, filter)?;
        Ok(EventSubscription::new(id, event_type.to_string()))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ExtensionContext::new("test-extension".to_string());
        assert_eq!(ctx.extension_id(), "test-extension");
    }

    #[test]
    fn test_capability_constants() {
        assert_eq!(capabilities::DEVICE_METRICS_READ, "device_metrics_read");
        assert_eq!(capabilities::DEVICE_METRICS_WRITE, "device_metrics_write");
        assert_eq!(capabilities::DEVICE_CONTROL, "device_control");
        assert_eq!(capabilities::STORAGE_QUERY, "storage_query");
        assert_eq!(capabilities::EVENT_PUBLISH, "event_publish");
        assert_eq!(capabilities::EVENT_SUBSCRIBE, "event_subscribe");
        assert_eq!(capabilities::TELEMETRY_HISTORY, "telemetry_history");
        assert_eq!(capabilities::METRICS_AGGREGATE, "metrics_aggregate");
        assert_eq!(capabilities::EXTENSION_CALL, "extension_call");
        assert_eq!(capabilities::AGENT_TRIGGER, "agent_trigger");
        assert_eq!(capabilities::RULE_TRIGGER, "rule_trigger");
    }

    #[test]
    fn test_device_read_params() {
        let ctx = ExtensionContext::new("test".to_string());
        // We can't actually call the host, but we can verify the params construction
        let params = serde_json::json!({
            "device_id": "device-1",
            "metric": "temperature",
        });

        assert_eq!(params["device_id"], "device-1");
        assert_eq!(params["metric"], "temperature");
    }

    #[test]
    fn test_device_write_params() {
        let params = serde_json::json!({
            "device_id": "device-1",
            "key": "status",
            "value": "active",
            "is_virtual": true,
        });

        assert_eq!(params["is_virtual"], true);
    }

    #[test]
    fn test_query_telemetry_params() {
        let params = serde_json::json!({
            "device_id": "device-1",
            "metric": "temperature",
            "start": 1000i64,
            "end": 2000i64,
        });

        assert_eq!(params["start"], 1000);
        assert_eq!(params["end"], 2000);
    }

    #[test]
    fn test_aggregate_params() {
        let params = serde_json::json!({
            "device_id": "device-1",
            "metric": "temp",
            "aggregation": "avg",
            "start": 1000i64,
            "end": 2000i64,
        });

        assert_eq!(params["aggregation"], "avg");
    }

    #[test]
    fn test_event_subscription() {
        let sub = EventSubscription::new(1, "device_changed".to_string());
        assert_eq!(sub.id, 1);
        assert_eq!(sub.event_type(), "device_changed");
    }
}
