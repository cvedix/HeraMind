use async_trait::async_trait;
use heramind_extension_sdk::{
    heramind_export, Extension, ExtensionCommand, ExtensionError, ExtensionMetadata,
    ExtensionMetricValue, MetricDataType, MetricDescriptor, ParamMetricValue,
    ParameterDefinition, Result,
};
use serde_json::json;

pub struct TestExtension;

impl TestExtension {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Extension for TestExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        static META: std::sync::OnceLock<ExtensionMetadata> = std::sync::OnceLock::new();
        META.get_or_init(|| {
            ExtensionMetadata::new("test-extension", "TestExtension", "0.1.0")
                .with_description("A HeraMind extension")
        })
    }

    fn commands(&self) -> Vec<ExtensionCommand> {
        vec![ExtensionCommand {
            name: "hello".to_string(),
            display_name: "Hello".to_string(),
            description: "Returns a greeting".to_string(),
            payload_template: String::new(),
            parameters: vec![ParameterDefinition {
                name: "name".to_string(),
                display_name: "Name".to_string(),
                description: "Who to greet".to_string(),
                param_type: MetricDataType::String,
                required: true,
                default_value: None,
                min: None,
                max: None,
                options: Vec::new(),
            }],
            fixed_values: std::collections::HashMap::new(),
            samples: vec![json!({"name": "world"})],
            parameter_groups: Vec::new(),
        }]
    }

    fn metrics(&self) -> Vec<MetricDescriptor> {
        vec![MetricDescriptor {
            name: "invocations".to_string(),
            display_name: "Invocations".to_string(),
            data_type: MetricDataType::Integer,
            unit: "count".to_string(),
            min: Some(0.0),
            max: None,
            required: false,
        }]
    }

    async fn execute_command(
        &self,
        command: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        match command {
            "hello" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("world");
                Ok(json!({"greeting": format!("Hello, {}!", name)}))
            }
            _ => Err(ExtensionError::CommandNotFound(command.to_string())),
        }
    }

    fn produce_metrics(&self) -> Result<Vec<ExtensionMetricValue>> {
        Ok(vec![ExtensionMetricValue::new(
            "invocations",
            ParamMetricValue::Integer(1),
        )])
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

heramind_export!(TestExtension);
