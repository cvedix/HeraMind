//! Tests for Extension SDK Capabilities Module
//!
//! Tests cover:
//! - Device capability types
//! - Event capability types
//! - Agent capability types
//! - Rule capability types
//! - Extension capability types

use heramind_extension_sdk::*;
use serde_json::json;

// ============================================================================
// Device Capability Type Tests
// ============================================================================

#[test]
fn test_sdk_metric_data_type_variants() {
    // Test all variants
    let float = SdkMetricDataType::Float;
    let integer = SdkMetricDataType::Integer;
    let boolean = SdkMetricDataType::Boolean;
    let string = SdkMetricDataType::String;
    let binary = SdkMetricDataType::Binary;

    // Verify serialization
    assert!(serde_json::to_string(&float).unwrap().contains("float"));
    assert!(serde_json::to_string(&integer).unwrap().contains("integer"));
    assert!(serde_json::to_string(&boolean).unwrap().contains("boolean"));
    assert!(serde_json::to_string(&string).unwrap().contains("string"));
    assert!(serde_json::to_string(&binary).unwrap().contains("binary"));
}

#[test]
fn test_sdk_metric_value_conversions() {
    // Float conversion
    let float_val: SdkMetricValue = 1.234.into();
    match float_val {
        SdkMetricValue::Float(v) => assert!((v - 1.234).abs() < 0.001),
        _ => panic!("Expected Float"),
    }

    // Integer conversion
    let int_val: SdkMetricValue = 42i64.into();
    match int_val {
        SdkMetricValue::Integer(v) => assert_eq!(v, 42),
        _ => panic!("Expected Integer"),
    }

    // Boolean conversion
    let bool_val: SdkMetricValue = true.into();
    match bool_val {
        SdkMetricValue::Boolean(v) => assert!(v),
        _ => panic!("Expected Boolean"),
    }

    // String conversion
    let str_val: SdkMetricValue = "hello".to_string().into();
    match str_val {
        SdkMetricValue::String(v) => assert_eq!(v, "hello"),
        _ => panic!("Expected String"),
    }

    // Binary conversion
    let bin_val: SdkMetricValue = vec![1u8, 2, 3].into();
    match bin_val {
        SdkMetricValue::Binary(v) => assert_eq!(v, vec![1, 2, 3]),
        _ => panic!("Expected Binary"),
    }
}

#[test]
fn test_sdk_metric_value_null() {
    let null_val = SdkMetricValue::Null;
    let json = serde_json::to_string(&null_val).unwrap();
    assert_eq!(json, "null");
}

// ============================================================================
// Device Metric Definition Tests
// ============================================================================

#[test]
fn test_sdk_metric_definition_temperature() {
    let metric = SdkMetricDefinition::new("temperature", "Temperature", SdkMetricDataType::Float)
        .with_unit("°C")
        .with_min(-40.0)
        .with_max(100.0)
        .with_required(true);

    assert_eq!(metric.name, "temperature");
    assert_eq!(metric.display_name, "Temperature");
    assert_eq!(metric.data_type, SdkMetricDataType::Float);
    assert_eq!(metric.unit, "°C");
    assert_eq!(metric.min, Some(-40.0));
    assert_eq!(metric.max, Some(100.0));
    assert!(metric.required);
}

#[test]
fn test_sdk_metric_definition_humidity() {
    let metric = SdkMetricDefinition::new("humidity", "Humidity", SdkMetricDataType::Float)
        .with_unit("%")
        .with_min(0.0)
        .with_max(100.0);

    assert_eq!(metric.name, "humidity");
    assert_eq!(metric.unit, "%");
    assert_eq!(metric.min, Some(0.0));
    assert_eq!(metric.max, Some(100.0));
}

#[test]
fn test_sdk_metric_definition_status() {
    let metric = SdkMetricDefinition::new("status", "Device Status", SdkMetricDataType::String);

    assert_eq!(metric.name, "status");
    assert_eq!(metric.data_type, SdkMetricDataType::String);
    assert!(!metric.required);
}

#[test]
fn test_sdk_metric_definition_serialization() {
    let metric =
        SdkMetricDefinition::new("power", "Power", SdkMetricDataType::Boolean).with_required(true);

    let json = serde_json::to_string(&metric).unwrap();
    let parsed: SdkMetricDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "power");
    assert_eq!(parsed.data_type, SdkMetricDataType::Boolean);
    assert!(parsed.required);
}

// ============================================================================
// Extension Metric Value Tests
// ============================================================================

#[test]
fn test_sdk_extension_metric_value_float() {
    let metric = SdkExtensionMetricValue::new("temperature", SdkMetricValue::Float(25.5));

    assert_eq!(metric.name, "temperature");
    match metric.value {
        SdkMetricValue::Float(v) => assert!((v - 25.5).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
    assert!(metric.timestamp > 0);
}

#[test]
fn test_sdk_extension_metric_value_integer() {
    let metric = SdkExtensionMetricValue::new("counter", SdkMetricValue::Integer(100));

    assert_eq!(metric.name, "counter");
    match metric.value {
        SdkMetricValue::Integer(v) => assert_eq!(v, 100),
        _ => panic!("Expected Integer"),
    }
}

#[test]
fn test_sdk_extension_metric_value_with_timestamp() {
    let ts = 1700000000000i64;
    let metric = SdkExtensionMetricValue::with_timestamp(
        "custom_metric",
        SdkMetricValue::String("active".to_string()),
        ts,
    );

    assert_eq!(metric.timestamp, ts);
}

#[test]
fn test_sdk_extension_metric_value_serialization() {
    let metric = SdkExtensionMetricValue::new("status", SdkMetricValue::Boolean(true));

    let json = serde_json::to_string(&metric).unwrap();
    let parsed: SdkExtensionMetricValue = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "status");
    match parsed.value {
        SdkMetricValue::Boolean(v) => assert!(v),
        _ => panic!("Expected Boolean"),
    }
}

// ============================================================================
// Parameter Definition Tests
// ============================================================================

#[test]
fn test_sdk_parameter_definition_required() {
    let param = SdkParameterDefinition::new("device_id", SdkMetricDataType::String)
        .with_display_name("Device ID")
        .with_description("The target device identifier");

    assert_eq!(param.name, "device_id");
    assert_eq!(param.display_name, "Device ID");
    assert_eq!(param.description, "The target device identifier");
    assert!(param.required);
}

#[test]
fn test_sdk_parameter_definition_optional_with_default() {
    let param = SdkParameterDefinition::new("timeout", SdkMetricDataType::Integer)
        .with_display_name("Timeout")
        .with_default(SdkMetricValue::Integer(30));

    assert!(!param.required);
    match &param.default_value {
        Some(SdkMetricValue::Integer(v)) => assert_eq!(*v, 30),
        _ => panic!("Expected Integer(30)"),
    }
}

#[test]
fn test_sdk_parameter_definition_with_range() {
    let param = SdkParameterDefinition::new("brightness", SdkMetricDataType::Integer)
        .with_display_name("Brightness")
        .optional();

    assert!(!param.required);
}

#[test]
fn test_sdk_parameter_definition_serialization() {
    let param = SdkParameterDefinition::new("mode", SdkMetricDataType::String)
        .with_display_name("Mode")
        .with_description("Operation mode")
        .optional();

    let json = serde_json::to_string(&param).unwrap();
    let parsed: SdkParameterDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "mode");
    assert!(!parsed.required);
}

// ============================================================================
// Command Definition Tests
// ============================================================================

#[test]
fn test_sdk_command_definition_simple() {
    let cmd = SdkCommandDefinition::new("turn_on");

    assert_eq!(cmd.name, "turn_on");
    assert!(cmd.parameters.is_empty());
    assert!(cmd.samples.is_empty());
}

#[test]
fn test_sdk_command_definition_with_params() {
    let cmd = SdkCommandDefinition::new("set_temperature")
        .with_description("Set the target temperature")
        .param(
            SdkParameterDefinition::new("target", SdkMetricDataType::Float)
                .with_display_name("Target Temperature"),
        )
        .param(
            SdkParameterDefinition::new("unit", SdkMetricDataType::String)
                .with_display_name("Unit")
                .with_default(SdkMetricValue::String("celsius".to_string())),
        );

    assert_eq!(cmd.name, "set_temperature");
    assert_eq!(cmd.parameters.len(), 2);
    assert!(cmd.parameters[0].required);
    assert!(!cmd.parameters[1].required);
}

#[test]
fn test_sdk_command_definition_serialization() {
    let cmd = SdkCommandDefinition::new("toggle")
        .with_description("Toggle the device state")
        .param(
            SdkParameterDefinition::new("force", SdkMetricDataType::Boolean)
                .with_display_name("Force")
                .optional(),
        );

    let json = serde_json::to_string(&cmd).unwrap();
    let parsed: SdkCommandDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "toggle");
    assert_eq!(parsed.description, "Toggle the device state");
    assert_eq!(parsed.parameters.len(), 1);
}

// ============================================================================
// Error Type Tests
// ============================================================================

#[test]
fn test_sdk_error_display() {
    let errors = vec![
        (
            SdkExtensionError::CommandNotFound("test".into()),
            "Command not found",
        ),
        (
            SdkExtensionError::InvalidArguments("bad args".into()),
            "Invalid arguments",
        ),
        (
            SdkExtensionError::ExecutionFailed("crash".into()),
            "Execution failed",
        ),
        (SdkExtensionError::Timeout("30s".into()), "Timeout"),
        (SdkExtensionError::NotFound("device".into()), "Not found"),
        (
            SdkExtensionError::InvalidFormat("json".into()),
            "Invalid format",
        ),
        (SdkExtensionError::LoadFailed("lib".into()), "Load failed"),
        (
            SdkExtensionError::SecurityError("denied".into()),
            "Security error",
        ),
        (
            SdkExtensionError::NotSupported("wasm".into()),
            "Not supported",
        ),
        (
            SdkExtensionError::ConfigurationError("bad config".into()),
            "Configuration error",
        ),
        (
            SdkExtensionError::InternalError("panic".into()),
            "Internal error",
        ),
        (SdkExtensionError::Other("custom".into()), "Error"),
    ];

    for (err, expected) in errors {
        let msg = err.to_string();
        assert!(
            msg.contains(expected),
            "Expected '{}' in '{}'",
            expected,
            msg
        );
    }
}

#[test]
fn test_sdk_error_from_serde_json() {
    let bad_json = "not valid json {";
    let result: std::result::Result<serde_json::Value, serde_json::Error> =
        serde_json::from_str(bad_json);
    let err: SdkExtensionError = result.unwrap_err().into();

    assert!(matches!(err, SdkExtensionError::InvalidFormat(_)));
}

// ============================================================================
// Frontend Manifest Tests
// ============================================================================

#[test]
fn test_frontend_manifest_builder() {
    let manifest = FrontendManifestBuilder::new("com.example.extension", "1.0.0")
        .entrypoint("dist/index.js")
        .style_entrypoint("dist/styles.css")
        .card("main-card", "Main Card")
        .widget("status-widget", "Status Widget")
        .panel("settings-panel", "Settings Panel")
        .dependency("react", "^18.0.0")
        .build();

    assert_eq!(manifest.id, "com.example.extension");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.entrypoint, "dist/index.js");
    assert_eq!(
        manifest.style_entrypoint,
        Some("dist/styles.css".to_string())
    );
    assert_eq!(manifest.components.len(), 3);
    assert_eq!(manifest.dependencies.len(), 1);
}

#[test]
fn test_frontend_component_types() {
    let types = vec![
        FrontendComponentType::Card,
        FrontendComponentType::Widget,
        FrontendComponentType::Panel,
        FrontendComponentType::Dialog,
        FrontendComponentType::Settings,
    ];

    for ct in types {
        let json = serde_json::to_string(&ct).unwrap();
        let parsed: FrontendComponentType = serde_json::from_str(&json).unwrap();
        assert_eq!(ct, parsed);
    }
}

#[test]
fn test_component_size() {
    let size = ComponentSize::new(400, 300);

    assert_eq!(size.width, 400);
    assert_eq!(size.height, 300);

    let json = serde_json::to_string(&size).unwrap();
    let parsed: ComponentSize = serde_json::from_str(&json).unwrap();
    assert_eq!(size.width, parsed.width);
    assert_eq!(size.height, parsed.height);
}

#[test]
fn test_i18n_config() {
    let i18n = I18nConfig {
        default_language: "en".to_string(),
        supported_languages: vec!["en".to_string(), "zh".to_string(), "ja".to_string()],
        resources_path: Some("locales".to_string()),
    };

    let json = serde_json::to_string(&i18n).unwrap();
    let parsed: I18nConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.default_language, "en");
    assert_eq!(parsed.supported_languages.len(), 3);
}

// ============================================================================
// Argument Parser Tests
// ============================================================================

#[test]
fn test_arg_parser_string() {
    let args = json!({"name": "test", "value": 42});
    let parser = ArgParser::new(&args);

    assert_eq!(parser.get_string("name").unwrap(), "test");
    assert!(parser.get_string("missing").is_err());
    assert_eq!(parser.get_optional_string("name"), Some("test".to_string()));
    assert_eq!(parser.get_optional_string("missing"), None);
}

#[test]
fn test_arg_parser_numbers() {
    let args = json!({"int": 42, "float": 1.234});
    let parser = ArgParser::new(&args);

    assert_eq!(parser.get_i64("int").unwrap(), 42);
    assert_eq!(parser.get_f64("float").unwrap(), 1.234);
    assert_eq!(parser.get_optional_i64("int"), Some(42));
    assert_eq!(parser.get_optional_f64("float"), Some(1.234));
}

#[test]
fn test_arg_parser_bool() {
    let args = json!({"enabled": true, "disabled": false});
    let parser = ArgParser::new(&args);

    assert!(parser.get_bool("enabled").unwrap());
    assert!(!parser.get_bool("disabled").unwrap());
    assert_eq!(parser.get_optional_bool("enabled"), Some(true));
}

#[test]
fn test_arg_parser_complex() {
    let args = json!({
        "config": {"key": "value"},
        "items": [1, 2, 3],
    });
    let parser = ArgParser::new(&args);

    let obj = parser.get_object("config").unwrap();
    assert_eq!(obj.get("key").unwrap().as_str().unwrap(), "value");

    let arr = parser.get_array("items").unwrap();
    assert_eq!(arr.len(), 3);
}

#[test]
fn test_arg_parser_parse_struct() {
    #[derive(serde::Deserialize)]
    struct TestConfig {
        name: String,
        count: i64,
        enabled: bool,
    }

    let args = json!({
        "name": "test",
        "count": 10,
        "enabled": true,
    });
    let parser = ArgParser::new(&args);

    let config: TestConfig = parser.parse().unwrap();
    assert_eq!(config.name, "test");
    assert_eq!(config.count, 10);
    assert!(config.enabled);
}

// ============================================================================
// Parameter Group Tests
// ============================================================================

#[test]
fn test_sdk_parameter_group() {
    let group = SdkParameterGroup {
        name: "advanced".to_string(),
        display_name: "Advanced Settings".to_string(),
        description: "Advanced configuration options".to_string(),
        parameters: vec!["timeout".to_string(), "retries".to_string()],
    };

    let json = serde_json::to_string(&group).unwrap();
    let parsed: SdkParameterGroup = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.name, "advanced");
    assert_eq!(parsed.parameters.len(), 2);
}

#[test]
fn test_command_with_parameter_groups() {
    let cmd = SdkCommandDefinition::new("configure")
        .param(SdkParameterDefinition::new(
            "timeout",
            SdkMetricDataType::Integer,
        ))
        .param(SdkParameterDefinition::new(
            "retries",
            SdkMetricDataType::Integer,
        ));

    let json = serde_json::to_string(&cmd).unwrap();
    let parsed: SdkCommandDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.parameters.len(), 2);
}
