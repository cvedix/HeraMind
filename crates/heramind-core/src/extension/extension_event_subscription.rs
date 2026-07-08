//!
//! Extension Event Subscription Service
//!
//! This service manages extension subscriptions to the EventBus.
//! Extensions can subscribe to specific event types and receive events directly
//! from the EventBus, eliminating the need for a separate event dispatcher.
//!
//! # Automatic Event Conversion
//!
//! All HeraMindEvent types are automatically converted to a standardized JSON format
//! and forwarded to subscribed extensions. No manual maintenance is required when
//! new event types are added to the system.
//!
//! # Extension Event Subscription
//!
//! Extensions declare their event subscriptions via the `event_subscriptions()` method:
//! - Return an empty slice `&[]` to subscribe to no events
//! - Return `&["all"]` to subscribe to all events
//! - Return specific event types like `&["DeviceMetric", "AgentExecutionStarted"]`
//!
//! Extensions receive events via the `handle_event()` method and can decide what
//! actions to take based on the event type and payload.

use std::sync::Arc;

use serde_json::Value;
use tracing::{info, trace};

use crate::event::HeraMindEvent;
use crate::eventbus::EventBus;
use crate::extension::event_dispatcher::EventDispatcher;

/// Extension event subscription service.
///
/// Manages extension subscriptions to the EventBus and forwards events
/// to subscribed extensions. This eliminates the need for a separate
/// event dispatcher by directly using EventBus subscriptions.
///
/// # Automatic Event Conversion
///
/// All HeraMindEvent types are automatically converted to a standardized JSON format:
/// ```json
/// {
///   "event_type": "DeviceMetric",
///   "payload": { ... event data ... },
///   "timestamp": 1234567890
/// }
/// ```
///
/// Extensions receive this format and can parse it to extract relevant information.
pub struct ExtensionEventSubscriptionService {
    event_bus: Arc<EventBus>,
    event_dispatcher: Arc<EventDispatcher>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl ExtensionEventSubscriptionService {
    /// Create a new extension event subscription service.
    pub fn new(event_bus: Arc<EventBus>, event_dispatcher: Arc<EventDispatcher>) -> Self {
        Self {
            event_bus,
            event_dispatcher,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start the service.
    ///
    /// This subscribes to all events from the EventBus and forwards them
    /// to extensions that have registered subscriptions via EventCapabilityProvider.
    pub fn start(&self) -> Arc<std::sync::atomic::AtomicBool> {
        if self
            .running
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
        {
            let _running = self.running.clone();
            let event_dispatcher = self.event_dispatcher.clone();

            // Subscribe to EventBus BEFORE spawning the task
            // This ensures we don't miss any events due to race conditions
            let mut rx = self.event_bus.subscribe();

            info!("Extension event subscription service started - subscribing to EventBus");

            tokio::spawn(async move {
                while _running.load(std::sync::atomic::Ordering::SeqCst) {
                    match rx.recv().await {
                        Some((event, _metadata)) => {
                            // Convert HeraMindEvent to extension format and dispatch (async)
                            Self::handle_event(&event_dispatcher, event).await;
                        }
                        None => {
                            info!("Extension event subscription service - EventBus closed");
                            break;
                        }
                    }
                }

                info!("Extension event subscription service stopped");
            });
        }

        self.running.clone()
    }

    /// Stop the service.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Handle an event from EventBus and dispatch to subscribed extensions.
    ///
    /// This converts HeraMindEvent to a standardized JSON format that extensions
    /// can understand, then forwards it to the EventDispatcher for distribution.
    async fn handle_event(event_dispatcher: &EventDispatcher, event: HeraMindEvent) {
        // Prevent feedback loops: skip re-dispatching virtual DeviceMetric events
        // to extensions. These are metrics written by extensions via device_metrics_write.
        // Without this filter, extensions would receive their own virtual metric writes
        // and potentially re-process them, creating infinite loops.
        if event.is_virtual_metric() {
            trace!(
                "Skipping virtual DeviceMetric dispatch to extensions (feedback loop prevention)"
            );
            return;
        }

        // Prevent feedback loops: skip dispatching ExtensionOutput events back to
        // the extension that produced them. Extensions subscribing to "all" or "Extension"
        // prefix would receive their own output events, potentially causing infinite loops.
        if let HeraMindEvent::ExtensionOutput {
            ref extension_id, ..
        } = event
        {
            trace!(
                extension_id = %extension_id,
                "Dispatching ExtensionOutput event, excluding source extension"
            );
            let (event_type, payload) = Self::convert_to_extension_format(&event);
            event_dispatcher
                .dispatch_event_excluding(&event_type, payload, extension_id)
                .await;
            return;
        }

        // Convert HeraMindEvent to extension format (automatic)
        let (event_type, payload) = Self::convert_to_extension_format(&event);

        if event_type == "AgentStreamChunk" {
            info!(event_type = %event_type, "Subscription service forwarding AgentStreamChunk to dispatcher");
        } else {
            trace!(event_type = %event_type, "Forwarding event from EventBus to extensions");
        }

        // Dispatch to extensions via EventDispatcher (async)
        event_dispatcher.dispatch_event(&event_type, payload).await;
    }

    /// Convert HeraMindEvent to extension format (automatic).
    ///
    /// This automatically converts ANY HeraMindEvent to a standardized JSON format.
    /// No manual maintenance is required when new event types are added.
    ///
    /// # Format
    ///
    /// The output format is:
    /// ```json
    /// {
    ///   "event_type": "DeviceMetric",
    ///   "payload": { ... serialized event data ... },
    ///   "timestamp": 1234567890
    /// }
    ///
    /// # Event Type Names
    ///
    /// Event type names are derived from the HeraMindEvent enum variant names:
    /// - `DeviceMetric` → "DeviceMetric"
    /// - `AgentExecutionStarted` → "AgentExecutionStarted"
    /// - `Custom { event_type: "my_event", ... }` → "my_event"
    ///
    /// # Automatic Support
    ///
    /// This method automatically supports ALL HeraMindEvent types, including:
    /// - Device events (DeviceMetric, DeviceOnline, DeviceOffline, etc.)
    /// - Rule events (RuleEvaluated, RuleTriggered, RuleExecuted)
    /// - Alert/Message events (AlertCreated, MessageCreated, etc.)
    /// - Agent events (AgentExecutionStarted, AgentThinking, etc.)
    /// - LLM events (LlmDecisionProposed, LlmDecisionExecuted, etc.)
    /// - Tool execution events (ToolExecutionStart, ToolExecutionSuccess, etc.)
    /// - Extension events (ExtensionOutput, ExtensionLifecycle, etc.)
    /// - User events (UserMessage, LlmResponse)
    /// - Custom events (any custom event type)
    ///
    /// New event types added to HeraMindEvent are automatically supported without
    /// any code changes to this method.
    fn convert_to_extension_format(event: &HeraMindEvent) -> (String, Value) {
        // Get the event type name from the enum variant
        // Use type_name_owned() to get the actual event_type for Custom events
        let event_type = event.type_name_owned();

        // Serialize the entire event to JSON
        let event_json = match serde_json::to_value(event) {
            Ok(json) => json,
            Err(e) => {
                tracing::warn!(
                    event_type = %event_type,
                    error = %e,
                    "Failed to serialize event, using fallback format"
                );
                // Fallback: create a minimal payload
                serde_json::json!({
                    "error": format!("Failed to serialize event: {}", e),
                    "event_type": event_type,
                })
            }
        };

        // Extract the event data (remove the "type" field if present)
        let payload = if let Some(obj) = event_json.as_object() {
            // Remove the "type" field if it exists (it's redundant with event_type)
            let mut payload = obj.clone();
            payload.remove("type");
            serde_json::Value::Object(payload)
        } else {
            event_json
        };

        // Extract timestamp from the event if available
        let timestamp = extract_timestamp_from_event(&payload);

        // Create the standardized extension event format
        let extension_event = serde_json::json!({
            "event_type": event_type,
            "payload": payload,
            "timestamp": timestamp,
        });

        (event_type.to_string(), extension_event)
    }
}

/// Extract timestamp from an event payload.
///
/// This helper function tries to find a timestamp field in the event payload.
/// If no timestamp is found, it returns the current time.
fn extract_timestamp_from_event(payload: &Value) -> i64 {
    // Try common timestamp field names
    let timestamp_fields = ["timestamp", "time", "ts", "created_at", "occurred_at"];

    for field in &timestamp_fields {
        if let Some(ts) = payload.get(field) {
            if let Some(ts_i64) = ts.as_i64() {
                return ts_i64;
            }
        }
    }

    // No timestamp found, use current time
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::MetricValue;

    #[test]
    fn test_convert_device_metric() {
        let event = HeraMindEvent::DeviceMetric {
            device_id: "test-device".to_string(),
            metric: "temperature".to_string(),
            value: MetricValue::Float(25.5),
            timestamp: 1234567890,
            quality: Some(0.95),
            is_virtual: None,
        };

        let (event_type, payload) =
            ExtensionEventSubscriptionService::convert_to_extension_format(&event);

        assert_eq!(event_type, "DeviceMetric");
        assert_eq!(payload["event_type"], "DeviceMetric");
        assert_eq!(payload["payload"]["device_id"], "test-device");
        assert_eq!(payload["payload"]["metric"], "temperature");
        assert_eq!(payload["payload"]["value"], serde_json::json!(25.5));
        assert_eq!(payload["timestamp"], 1234567890);
    }

    #[test]
    fn test_convert_custom_event() {
        let event = HeraMindEvent::Custom {
            event_type: "my_custom_event".to_string(),
            data: serde_json::json!({"key": "value"}),
        };

        let (event_type, payload) =
            ExtensionEventSubscriptionService::convert_to_extension_format(&event);

        assert_eq!(event_type, "my_custom_event");
        assert_eq!(payload["event_type"], "my_custom_event");
        assert_eq!(payload["payload"]["data"]["key"], "value");
    }

    #[test]
    fn test_convert_agent_event() {
        let event = HeraMindEvent::AgentExecutionStarted {
            agent_id: "test-agent".to_string(),
            agent_name: "Test Agent".to_string(),
            execution_id: "exec-123".to_string(),
            trigger_type: "manual".to_string(),
            timestamp: 1234567890,
        };

        let (event_type, payload) =
            ExtensionEventSubscriptionService::convert_to_extension_format(&event);

        assert_eq!(event_type, "AgentExecutionStarted");
        assert_eq!(payload["event_type"], "AgentExecutionStarted");
        assert_eq!(payload["payload"]["agent_id"], "test-agent");
        assert_eq!(payload["payload"]["execution_id"], "exec-123");
    }
}
