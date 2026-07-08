//! Event Capabilities (Unified for Native and WASM)

use serde_json::{json, Value};

#[cfg(not(target_arch = "wasm32"))]
use crate::host::*;

#[cfg(target_arch = "wasm32")]
use crate::wasm::{capabilities, EventSubscription, ExtensionContext};

pub type CapabilityError = String;

#[cfg(not(target_arch = "wasm32"))]
pub type Context = ExtensionContext;

#[cfg(target_arch = "wasm32")]
pub type Context = crate::wasm::ExtensionContext;

// ============================================================================
// Event Handler Trait
// ============================================================================

/// Event handler callback
pub type EventHandler = Box<dyn Fn(&str, &Value) + Send + Sync>;

/// Global event handler registry (native extensions only)
#[cfg(not(target_arch = "wasm32"))]
static EVENT_HANDLER: std::sync::OnceLock<std::sync::Arc<std::sync::RwLock<Option<EventHandler>>>> =
    std::sync::OnceLock::new();

/// Register an event handler for native extensions
#[cfg(not(target_arch = "wasm32"))]
pub fn register_event_handler(handler: EventHandler) {
    let registry = EVENT_HANDLER.get_or_init(|| std::sync::Arc::new(std::sync::RwLock::new(None)));
    *registry.write().unwrap() = Some(handler);
}

/// Call the registered event handler (internal use)
#[cfg(not(target_arch = "wasm32"))]
pub fn call_event_handler(event_type: &str, payload: &Value) {
    if let Some(registry) = EVENT_HANDLER.get() {
        if let Some(handler) = registry.read().unwrap().as_ref() {
            handler(event_type, payload);
        }
    }
}

// ============================================================================
// Event Publish
// ============================================================================

/// Publish an event
#[cfg(not(target_arch = "wasm32"))]
pub async fn publish(
    context: &Context,
    event_type: &str,
    payload: &Value,
) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::EventPublish,
            &json!({"event_type": event_type, "payload": payload}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn publish(
    context: &Context,
    event_type: &str,
    payload: &Value,
) -> Result<Value, CapabilityError> {
    context.publish_event(event_type, payload)
}

// ============================================================================
// Event Subscribe
// ============================================================================

/// Subscribe to events
#[cfg(not(target_arch = "wasm32"))]
pub async fn subscribe(
    context: &Context,
    subscription: EventSubscription,
) -> Result<Value, CapabilityError> {
    let sub_value = serde_json::to_value(&subscription).map_err(|e| e.to_string())?;
    context
        .invoke_capability(
            ExtensionCapability::EventSubscribe,
            &json!({"subscription": sub_value}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn subscribe(
    context: &Context,
    event_type: &str,
    filter: Option<&Value>,
) -> Result<EventSubscription, CapabilityError> {
    context.subscribe_event(event_type, filter)
}

/// Poll for events (WASM only)
#[cfg(target_arch = "wasm32")]
pub fn poll_events(subscription: &EventSubscription) -> Result<Vec<Value>, CapabilityError> {
    subscription.poll().map_err(|e| e.to_string())
}

/// Unsubscribe from events
#[cfg(not(target_arch = "wasm32"))]
pub async fn unsubscribe(
    context: &Context,
    subscription_id: &str,
) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::EventSubscribe,
            &json!({
                "action": "unsubscribe",
                "subscription_id": subscription_id,
            }),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn unsubscribe(_subscription: EventSubscription) -> Result<(), CapabilityError> {
    // EventSubscription implements Drop, which will call host_event_unsubscribe
    Ok(())
}

/// List all subscriptions
#[cfg(not(target_arch = "wasm32"))]
pub async fn list_subscriptions(context: &Context) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::EventSubscribe,
            &json!({"action": "list"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn list_subscriptions(_context: &Context) -> Result<Value, CapabilityError> {
    Ok(json!({"subscriptions": []}))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {

    use serde_json::json;

    #[test]
    fn test_event_publish_params() {
        let event_type = "device_changed";
        let payload = json!({
            "device_id": "sensor-1",
            "old_state": "offline",
            "new_state": "online",
        });

        let params = json!({
            "event_type": event_type,
            "payload": payload,
        });

        assert_eq!(params["event_type"], "device_changed");
        assert_eq!(params["payload"]["device_id"], "sensor-1");
    }

    #[test]
    fn test_event_subscribe_params() {
        let event_type = "device_state_changed";
        let filter = json!({
            "device_id": "sensor-1",
        });

        let params = json!({
            "event_type": event_type,
            "filter": filter,
        });

        assert_eq!(params["event_type"], "device_state_changed");
        assert_eq!(params["filter"]["device_id"], "sensor-1");
    }

    #[test]
    fn test_unsubscribe_params() {
        let subscription_id = "sub-123";
        let params = json!({
            "action": "unsubscribe",
            "subscription_id": subscription_id,
        });

        assert_eq!(params["action"], "unsubscribe");
        assert_eq!(params["subscription_id"], "sub-123");
    }

    #[test]
    fn test_list_subscriptions_params() {
        let params = json!({"action": "list"});
        assert_eq!(params["action"], "list");
    }
}
