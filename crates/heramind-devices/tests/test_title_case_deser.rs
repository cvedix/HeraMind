use heramind_devices::mdl::MetricDataType;

#[test]
fn test_metric_data_type_title_case() {
    // Test lowercase (existing format)
    let json_lower = r#""float""#;
    let result: Result<MetricDataType, _> = serde_json::from_str(json_lower);
    assert!(result.is_ok(), "Failed to parse lowercase 'float'");
    assert_eq!(result.unwrap(), MetricDataType::Float);

    // Test Title Case (from heramind-device-types)
    let json_title = r#""Float""#;
    let result: Result<MetricDataType, _> = serde_json::from_str(json_title);
    assert!(result.is_ok(), "Failed to parse TitleCase 'Float'");
    assert_eq!(result.unwrap(), MetricDataType::Float);

    // Test String
    let json_string = r#""String""#;
    let result: Result<MetricDataType, _> = serde_json::from_str(json_string);
    assert!(result.is_ok(), "Failed to parse TitleCase 'String'");
    assert_eq!(result.unwrap(), MetricDataType::String);

    // Test Integer
    let json_integer = r#""Integer""#;
    let result: Result<MetricDataType, _> = serde_json::from_str(json_integer);
    assert!(result.is_ok(), "Failed to parse TitleCase 'Integer'");
    assert_eq!(result.unwrap(), MetricDataType::Integer);

    // Test Boolean
    let json_boolean = r#""Boolean""#;
    let result: Result<MetricDataType, _> = serde_json::from_str(json_boolean);
    assert!(result.is_ok(), "Failed to parse TitleCase 'Boolean'");
    assert_eq!(result.unwrap(), MetricDataType::Boolean);

    // Test Array
    let json_array = r#""Array""#;
    let result: Result<MetricDataType, _> = serde_json::from_str(json_array);
    assert!(result.is_ok(), "Failed to parse TitleCase 'Array'");

    // Test Binary
    let json_binary = r#""Binary""#;
    let result: Result<MetricDataType, _> = serde_json::from_str(json_binary);
    assert!(result.is_ok(), "Failed to parse TitleCase 'Binary'");
    assert_eq!(result.unwrap(), MetricDataType::Binary);

    println!("All TitleCase deserialization tests passed!");
}

fn main() {
    test_metric_data_type_title_case();
    println!("Test completed successfully!");
}
