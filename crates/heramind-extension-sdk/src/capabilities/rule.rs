//! Rule Capabilities (Unified for Native and WASM)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[cfg(not(target_arch = "wasm32"))]
use crate::host::*;

#[cfg(target_arch = "wasm32")]
use crate::wasm::{capabilities, ExtensionContext};

pub type CapabilityError = String;

#[cfg(not(target_arch = "wasm32"))]
pub type Context = ExtensionContext;

#[cfg(target_arch = "wasm32")]
pub type Context = crate::wasm::ExtensionContext;

/// Trigger a rule
#[cfg(not(target_arch = "wasm32"))]
pub async fn trigger(
    context: &Context,
    rule_id: &str,
    context_data: &Value,
) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::RuleTrigger,
            &json!({"rule_id": rule_id, "context": context_data}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn trigger(
    context: &Context,
    rule_id: &str,
    context_data: &Value,
) -> Result<Value, CapabilityError> {
    context.trigger_rule(rule_id, context_data)
}

/// Trigger with typed context
#[cfg(not(target_arch = "wasm32"))]
pub async fn trigger_typed<P>(
    context: &Context,
    rule_id: &str,
    context_data: &P,
) -> Result<Value, CapabilityError>
where
    P: Serialize,
{
    let ctx_json = serde_json::to_value(context_data).map_err(|e| e.to_string())?;
    trigger(context, rule_id, &ctx_json).await
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_typed<P>(
    context: &Context,
    rule_id: &str,
    context_data: &P,
) -> Result<Value, CapabilityError>
where
    P: Serialize,
{
    let ctx_json = serde_json::to_value(context_data).map_err(|e| e.to_string())?;
    trigger(context, rule_id, &ctx_json)
}

/// Trigger with typed response
#[cfg(not(target_arch = "wasm32"))]
pub async fn trigger_typed_response<P, R>(
    context: &Context,
    rule_id: &str,
    context_data: &P,
) -> Result<R, CapabilityError>
where
    P: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let result = trigger_typed(context, rule_id, context_data).await?;
    serde_json::from_value(result).map_err(|e| format!("Failed to parse response: {}", e))
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_typed_response<P, R>(
    context: &Context,
    rule_id: &str,
    context_data: &P,
) -> Result<R, CapabilityError>
where
    P: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let result = trigger_typed(context, rule_id, context_data)?;
    serde_json::from_value(result).map_err(|e| format!("Failed to parse response: {}", e))
}

/// Get rule definition
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_definition(context: &Context, rule_id: &str) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::RuleTrigger,
            &json!({"rule_id": rule_id, "action": "get_definition"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn get_definition(context: &Context, rule_id: &str) -> Result<Value, CapabilityError> {
    context.invoke_capability(
        capabilities::RULE_TRIGGER,
        &json!({"rule_id": rule_id, "action": "get_definition"}),
    )
}

/// List rules
#[cfg(not(target_arch = "wasm32"))]
pub async fn list(context: &Context) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(ExtensionCapability::RuleTrigger, &json!({"action": "list"}))
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn list(context: &Context) -> Result<Value, CapabilityError> {
    context.invoke_capability(capabilities::RULE_TRIGGER, &json!({"action": "list"}))
}

/// Enable a rule
#[cfg(not(target_arch = "wasm32"))]
pub async fn enable(context: &Context, rule_id: &str) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::RuleTrigger,
            &json!({"rule_id": rule_id, "action": "enable"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn enable(context: &Context, rule_id: &str) -> Result<Value, CapabilityError> {
    context.invoke_capability(
        capabilities::RULE_TRIGGER,
        &json!({"rule_id": rule_id, "action": "enable"}),
    )
}

/// Disable a rule
#[cfg(not(target_arch = "wasm32"))]
pub async fn disable(context: &Context, rule_id: &str) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::RuleTrigger,
            &json!({"rule_id": rule_id, "action": "disable"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn disable(context: &Context, rule_id: &str) -> Result<Value, CapabilityError> {
    context.invoke_capability(
        capabilities::RULE_TRIGGER,
        &json!({"rule_id": rule_id, "action": "disable"}),
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rule_trigger_params() {
        let rule_id = "alert-rule";
        let context_data = json!({
            "device_id": "sensor-1",
            "threshold": 80.0,
            "current_value": 85.0,
        });

        let params = json!({
            "rule_id": rule_id,
            "context": context_data,
        });

        assert_eq!(params["rule_id"], "alert-rule");
        assert_eq!(params["context"]["device_id"], "sensor-1");
        assert_eq!(params["context"]["threshold"], 80.0);
    }

    #[test]
    fn test_rule_enable_params() {
        let rule_id = "my-rule";
        let params = json!({
            "rule_id": rule_id,
            "action": "enable",
        });

        assert_eq!(params["rule_id"], "my-rule");
        assert_eq!(params["action"], "enable");
    }

    #[test]
    fn test_rule_disable_params() {
        let rule_id = "my-rule";
        let params = json!({
            "rule_id": rule_id,
            "action": "disable",
        });

        assert_eq!(params["action"], "disable");
    }

    #[test]
    fn test_rule_get_definition_params() {
        let rule_id = "alert-rule";
        let params = json!({
            "rule_id": rule_id,
            "action": "get_definition",
        });

        assert_eq!(params["action"], "get_definition");
    }

    #[test]
    fn test_rule_list_params() {
        let params = json!({"action": "list"});
        assert_eq!(params["action"], "list");
    }

    #[test]
    fn test_typed_context() {
        #[derive(Serialize)]
        struct AlertContext {
            device_id: String,
            threshold: f64,
            current_value: f64,
        }

        let ctx = AlertContext {
            device_id: "sensor-1".to_string(),
            threshold: 80.0,
            current_value: 85.5,
        };

        let ctx_json = serde_json::to_value(&ctx).unwrap();
        assert_eq!(ctx_json["device_id"], "sensor-1");
        assert_eq!(ctx_json["threshold"], 80.0);
        assert_eq!(ctx_json["current_value"], 85.5);
    }
}
