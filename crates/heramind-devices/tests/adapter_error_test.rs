//! Device Adapter Error Handling Tests
//!
//! Tests error handling in device adapters including:
//! - Connection failures
//! - Invalid configurations
//! - State transitions
//

use heramind_devices::adapter::AdapterError;

#[tokio::test]
async fn test_error_display_configuration() {
    // Test AdapterError Configuration variant
    let error = AdapterError::Configuration("test error".to_string());

    // Verify error message contains error text
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Configuration error: test error"));
}

#[tokio::test]
async fn test_error_display_connection() {
    // Test AdapterError Connection variant
    let error = AdapterError::Connection("connection failed".to_string());

    // Verify error message
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("connection failed"));
}

#[tokio::test]
async fn test_error_display_communication() {
    // Test AdapterError Communication variant
    let error = AdapterError::Communication("send failed".to_string());

    // Verify error message
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("send failed"));
}

#[tokio::test]
async fn test_error_display_stopped() {
    // Test AdapterError Stopped variant (the message is built-in via thiserror)
    let error = AdapterError::Stopped;

    // Verify error message
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("stopped"));
}

#[tokio::test]
async fn test_error_display_other() {
    // Test AdapterError Other variant with anyhow

    let inner_error = anyhow::anyhow!("test error");
    let error = AdapterError::Other(inner_error);

    // Verify error message
    let error_msg = format!("{:?}", error);
    assert!(error_msg.contains("test error"));
}
