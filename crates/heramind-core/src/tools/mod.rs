//! Core tool abstractions for HeraMind.
//!
//! This module defines the foundational traits for function calling
/// used by LLM agents.
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool category for grouping and organization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum ToolCategory {
    /// Device operations (control, query, configure)
    Device,
    /// Data operations (query, aggregate, export)
    Data,
    /// Analysis operations (trends, anomalies, predictions)
    Analysis,
    /// Rule operations (create, update, delete, enable/disable)
    Rule,
    /// Alert operations (query, acknowledge)
    Alert,
    /// AI Agent operations (query, execute, control, create)
    Agent,
    /// System operations (search, thinking)
    #[default]
    System,
    /// Configuration operations
    Config,
}

impl ToolCategory {
    /// Get category identifier for LLM prompts
    pub fn as_str(&self) -> &str {
        match self {
            ToolCategory::Device => "device",
            ToolCategory::Data => "data",
            ToolCategory::Analysis => "analysis",
            ToolCategory::Rule => "rule",
            ToolCategory::Alert => "alert",
            ToolCategory::Agent => "agent",
            ToolCategory::System => "system",
            ToolCategory::Config => "config",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            ToolCategory::Device => "Device Management",
            ToolCategory::Data => "Data Query",
            ToolCategory::Analysis => "Data Analysis",
            ToolCategory::Rule => "Rule Management",
            ToolCategory::Alert => "Alert Management",
            ToolCategory::Agent => "Agent Management",
            ToolCategory::System => "System Tools",
            ToolCategory::Config => "Configuration",
        }
    }
}

/// Usage scenario for tool guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageScenario {
    /// Scenario description
    pub description: String,
    /// Example user query that triggers this scenario
    pub example_query: String,
    /// Suggested tool call for this scenario
    pub suggested_call: Option<String>,
}

/// Tool relationship metadata for guiding LLM behavior.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolRelationships {
    /// Tools that should typically be called before this tool
    pub call_after: Vec<String>,
    /// Tools that can use this tool's output as input
    pub output_to: Vec<String>,
    /// Tools that are mutually exclusive with this tool
    pub exclusive_with: Vec<String>,
}

/// Result type for tool operations.
pub type Result<T> = std::result::Result<T, ToolError>;

/// Tool error types.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Invalid arguments provided.
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Tool execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Tool not found.
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Other error.
    #[error("Tool error: {0}")]
    Other(#[from] anyhow::Error),
}

// Note: Tool trait, ToolOutput, Parameter, ToolDefinition, ToolExample, DynTool, and ToolFactory
// have been removed from heramind-core. The canonical implementations are in crates/heramind-agent/src/toolkit/tool.rs
// with 55+ implementations. These types are re-exported from heramind-agent for backward compatibility.

/// Helper function to create a JSON object schema for parameters.
pub fn object_schema(properties: Value, required: Vec<String>) -> Value {
    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Helper function to create a simple property definition.
pub fn property(prop_type: &str, description: &str) -> Value {
    serde_json::json!({
        "type": prop_type,
        "description": description
    })
}

/// Helper function to create a string property.
pub fn string_property(description: &str) -> Value {
    property("string", description)
}

/// Helper function to create a number property.
pub fn number_property(description: &str) -> Value {
    property("number", description)
}

/// Helper function to create a boolean property.
pub fn boolean_property(description: &str) -> Value {
    property("boolean", description)
}

/// Helper function to create an array property.
pub fn array_property(item_type: &str, description: &str) -> Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "type": item_type
        },
        "description": description
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_helpers() {
        let schema = object_schema(
            serde_json::json!({
                "name": string_property("The name"),
                "age": number_property("The age")
            }),
            vec!["name".to_string()],
        );

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"]["type"] == "string");
    }
}
