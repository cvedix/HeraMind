//! Extension Call Capabilities (Unified for Native and WASM)

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

/// Call another extension
#[cfg(not(target_arch = "wasm32"))]
pub async fn call(
    context: &Context,
    extension_id: &str,
    command: &str,
    args: &Value,
) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::ExtensionCall,
            &json!({
                "extension_id": extension_id,
                "command": command,
                "args": args,
            }),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn call(
    context: &Context,
    extension_id: &str,
    command: &str,
    args: &Value,
) -> Result<Value, CapabilityError> {
    context.call_extension(extension_id, command, args)
}

/// Call with typed arguments
#[cfg(not(target_arch = "wasm32"))]
pub async fn call_typed<P>(
    context: &Context,
    extension_id: &str,
    command: &str,
    args: &P,
) -> Result<Value, CapabilityError>
where
    P: Serialize,
{
    let args_json = serde_json::to_value(args).map_err(|e| e.to_string())?;
    call(context, extension_id, command, &args_json).await
}

#[cfg(target_arch = "wasm32")]
pub fn call_typed<P>(
    context: &Context,
    extension_id: &str,
    command: &str,
    args: &P,
) -> Result<Value, CapabilityError>
where
    P: Serialize,
{
    let args_json = serde_json::to_value(args).map_err(|e| e.to_string())?;
    call(context, extension_id, command, &args_json)
}

/// Call with typed response
#[cfg(not(target_arch = "wasm32"))]
pub async fn call_typed_response<P, R>(
    context: &Context,
    extension_id: &str,
    command: &str,
    args: &P,
) -> Result<R, CapabilityError>
where
    P: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let result = call_typed(context, extension_id, command, args).await?;
    serde_json::from_value(result).map_err(|e| format!("Failed to parse response: {}", e))
}

#[cfg(target_arch = "wasm32")]
pub fn call_typed_response<P, R>(
    context: &Context,
    extension_id: &str,
    command: &str,
    args: &P,
) -> Result<R, CapabilityError>
where
    P: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let result = call_typed(context, extension_id, command, args)?;
    serde_json::from_value(result).map_err(|e| format!("Failed to parse response: {}", e))
}

/// Health check
#[cfg(not(target_arch = "wasm32"))]
pub async fn health_check(context: &Context, extension_id: &str) -> Result<bool, CapabilityError> {
    let result = context
        .invoke_capability(
            ExtensionCapability::ExtensionCall,
            &json!({"extension_id": extension_id, "action": "health_check"}),
        )
        .await
        .map_err(|e| e.to_string())?;

    result
        .get("healthy")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid response".to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn health_check(context: &Context, extension_id: &str) -> Result<bool, CapabilityError> {
    let result = context.invoke_capability(
        capabilities::EXTENSION_CALL,
        &json!({"extension_id": extension_id, "action": "health_check"}),
    )?;

    result
        .get("healthy")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid response".to_string())
}

/// List extensions
#[cfg(not(target_arch = "wasm32"))]
pub async fn list(context: &Context) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::ExtensionCall,
            &json!({"action": "list"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn list(context: &Context) -> Result<Value, CapabilityError> {
    context.invoke_capability(capabilities::EXTENSION_CALL, &json!({"action": "list"}))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extension_call_params() {
        let extension_id = "weather-extension";
        let command = "get_forecast";
        let args = json!({
            "location": "Beijing",
            "days": 3,
        });

        let params = json!({
            "extension_id": extension_id,
            "command": command,
            "args": args,
        });

        assert_eq!(params["extension_id"], "weather-extension");
        assert_eq!(params["command"], "get_forecast");
        assert_eq!(params["args"]["location"], "Beijing");
    }

    #[test]
    fn test_health_check_params() {
        let extension_id = "my-extension";
        let params = json!({
            "extension_id": extension_id,
            "action": "health_check",
        });

        assert_eq!(params["extension_id"], "my-extension");
        assert_eq!(params["action"], "health_check");
    }

    #[test]
    fn test_list_params() {
        let params = json!({"action": "list"});
        assert_eq!(params["action"], "list");
    }

    #[test]
    fn test_typed_args_serialization() {
        #[derive(Serialize)]
        struct ForecastArgs {
            location: String,
            days: u32,
        }

        let args = ForecastArgs {
            location: "Shanghai".to_string(),
            days: 5,
        };

        let args_json = serde_json::to_value(&args).unwrap();
        assert_eq!(args_json["location"], "Shanghai");
        assert_eq!(args_json["days"], 5);
    }

    #[test]
    fn test_typed_response_deserialization() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct ForecastResult {
            temperature: f64,
            condition: String,
        }

        let json = json!({
            "temperature": 25.5,
            "condition": "sunny",
        });

        let result: ForecastResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.temperature, 25.5);
        assert_eq!(result.condition, "sunny");
    }
}
