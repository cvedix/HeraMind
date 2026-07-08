//! Agent Capabilities (Unified for Native and WASM)

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

/// Trigger an agent
#[cfg(not(target_arch = "wasm32"))]
pub async fn trigger(
    context: &Context,
    agent_id: &str,
    input: &Value,
) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::AgentTrigger,
            &json!({"agent_id": agent_id, "input": input}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn trigger(context: &Context, agent_id: &str, input: &Value) -> Result<Value, CapabilityError> {
    context.trigger_agent(agent_id, input)
}

/// Trigger with typed input
#[cfg(not(target_arch = "wasm32"))]
pub async fn trigger_typed<P>(
    context: &Context,
    agent_id: &str,
    input: &P,
) -> Result<Value, CapabilityError>
where
    P: Serialize,
{
    let input_json = serde_json::to_value(input).map_err(|e| e.to_string())?;
    trigger(context, agent_id, &input_json).await
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_typed<P>(
    context: &Context,
    agent_id: &str,
    input: &P,
) -> Result<Value, CapabilityError>
where
    P: Serialize,
{
    let input_json = serde_json::to_value(input).map_err(|e| e.to_string())?;
    trigger(context, agent_id, &input_json)
}

/// Trigger with typed response
#[cfg(not(target_arch = "wasm32"))]
pub async fn trigger_typed_response<P, R>(
    context: &Context,
    agent_id: &str,
    input: &P,
) -> Result<R, CapabilityError>
where
    P: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let result = trigger_typed(context, agent_id, input).await?;
    serde_json::from_value(result).map_err(|e| format!("Failed to parse response: {}", e))
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_typed_response<P, R>(
    context: &Context,
    agent_id: &str,
    input: &P,
) -> Result<R, CapabilityError>
where
    P: Serialize,
    R: for<'de> Deserialize<'de>,
{
    let result = trigger_typed(context, agent_id, input)?;
    serde_json::from_value(result).map_err(|e| format!("Failed to parse response: {}", e))
}

/// Get agent status
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_status(context: &Context, agent_id: &str) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::AgentTrigger,
            &json!({"agent_id": agent_id, "action": "status"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn get_status(context: &Context, agent_id: &str) -> Result<Value, CapabilityError> {
    context.invoke_capability(
        capabilities::AGENT_TRIGGER,
        &json!({"agent_id": agent_id, "action": "status"}),
    )
}

/// Stop an agent
#[cfg(not(target_arch = "wasm32"))]
pub async fn stop(context: &Context, agent_id: &str) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::AgentTrigger,
            &json!({"agent_id": agent_id, "action": "stop"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn stop(context: &Context, agent_id: &str) -> Result<Value, CapabilityError> {
    context.invoke_capability(
        capabilities::AGENT_TRIGGER,
        &json!({"agent_id": agent_id, "action": "stop"}),
    )
}

/// List agents
#[cfg(not(target_arch = "wasm32"))]
pub async fn list(context: &Context) -> Result<Value, CapabilityError> {
    context
        .invoke_capability(
            ExtensionCapability::AgentTrigger,
            &json!({"action": "list"}),
        )
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn list(context: &Context) -> Result<Value, CapabilityError> {
    context.invoke_capability(capabilities::AGENT_TRIGGER, &json!({"action": "list"}))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_trigger_params() {
        let agent_id = "analyzer-agent";
        let input = json!({
            "query": "analyze temperature trend",
            "time_range": "24h",
        });

        let params = json!({
            "agent_id": agent_id,
            "input": input,
        });

        assert_eq!(params["agent_id"], "analyzer-agent");
        assert_eq!(params["input"]["query"], "analyze temperature trend");
    }

    #[test]
    fn test_agent_status_params() {
        let agent_id = "my-agent";
        let params = json!({
            "agent_id": agent_id,
            "action": "status",
        });

        assert_eq!(params["agent_id"], "my-agent");
        assert_eq!(params["action"], "status");
    }

    #[test]
    fn test_agent_stop_params() {
        let agent_id = "running-agent";
        let params = json!({
            "agent_id": agent_id,
            "action": "stop",
        });

        assert_eq!(params["action"], "stop");
    }

    #[test]
    fn test_agent_list_params() {
        let params = json!({"action": "list"});
        assert_eq!(params["action"], "list");
    }

    #[test]
    fn test_typed_input() {
        #[derive(Serialize)]
        struct AnalyzeInput {
            query: String,
            time_range: String,
        }

        let input = AnalyzeInput {
            query: "analyze trends".to_string(),
            time_range: "7d".to_string(),
        };

        let input_json = serde_json::to_value(&input).unwrap();
        assert_eq!(input_json["query"], "analyze trends");
        assert_eq!(input_json["time_range"], "7d");
    }

    #[test]
    fn test_typed_output() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct AgentResponse {
            status: String,
            result: Value,
        }

        let json = json!({
            "status": "completed",
            "result": {"trend": "increasing"},
        });

        let response: AgentResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.status, "completed");
    }
}
