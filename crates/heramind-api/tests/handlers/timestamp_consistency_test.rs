//! Test timestamp consistency for telemetry queries
//!
//! This test verifies that:
//! 1. Data written with second-level timestamps can be queried
//! 2. Query time ranges match the storage keys
//! 3. No data loss due to timestamp unit mismatches

use heramind_api::tests::common::*;
use heramind_devices::MetricValue;
use serde_json::json;

#[tokio::test]
async fn test_telemetry_timestamp_consistency() {
    let (state, _auth) = setup_test_state().await;

    // Create a test device
    let device_id = "test-timestamp-device";
    create_test_device(&state, device_id).await;

    // Get current time in seconds (as used in storage)
    let now = chrono::Utc::now().timestamp();
    let metric_name = "temperature";
    let test_value = 25.5;

    // Write data point with second-level timestamp
    // (This is how MQTT adapter writes data)
    let storage = state.devices.telemetry.clone();
    let data_point = heramind_devices::telemetry::DataPoint {
        timestamp: now, // ← Second-level timestamp
        value: MetricValue::Float(test_value),
        quality: Some(1.0),
    };

    storage
        .write(device_id, metric_name, data_point)
        .await
        .expect("Failed to write telemetry");

    // Query using second-level timestamps
    // (This is how the API handler should query after the fix)
    let start = now - 60; // 1 minute ago
    let end = now + 60;   // 1 minute in future

    let points = storage
        .query(device_id, metric_name, start, end)
        .await
        .expect("Failed to query telemetry");

    // Verify we got the data back
    assert_eq!(points.len(), 1, "Should find exactly 1 data point");
    assert_eq!(points[0].timestamp, now, "Timestamp should match");
    assert_eq!(
        points[0].value,
        MetricValue::Float(test_value),
        "Value should match"
    );
}

#[tokio::test]
async fn test_api_handler_timestamp_consistency() {
    let (state, auth) = setup_test_state().await;

    // Create a test device
    let device_id = "test-api-timestamp-device";
    create_test_device(&state, device_id).await;

    // Write test data
    let now = chrono::Utc::now().timestamp();
    let storage = state.devices.telemetry.clone();
    let data_point = heramind_devices::telemetry::DataPoint {
        timestamp: now,
        value: MetricValue::Integer(100),
        quality: Some(1.0),
    };

    storage
        .write(device_id, "counter", data_point)
        .await
        .expect("Failed to write telemetry");

    // Query via API handler with second-level timestamps
    let app = test_app(state);
    let start = now - 60;
    let end = now + 60;

    let response = app
        .oneshot(
            reqwest::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/devices/{}/telemetry?metric=counter&start={}&end={}",
                    device_id, start, end
                ))
                .header("Authorization", format!("Bearer {}", auth.token))
                .body("{}".to_string())
                .unwrap()
                .try_into()
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = hyper::body::to_bytes(response.into_body())
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response contains data
    let data = json["data"]["counter"].as_array().expect("Should have data array");
    assert_eq!(data.len(), 1, "Should have 1 data point");

    let point = &data[0];
    assert_eq!(point["timestamp"], now, "Timestamp should match");
    assert_eq!(point["value"], 100, "Value should match");
}

#[tokio::test]
async fn test_virtual_metric_timestamp_consistency() {
    let (state, _auth) = setup_test_state().await;

    // Create a test device
    let device_id = "test-virtual-timestamp-device";
    create_test_device(&state, device_id).await;

    // Write virtual metric with second-level timestamp
    let now = chrono::Utc::now().timestamp();
    let storage = state.devices.telemetry.clone();
    let data_point = heramind_devices::telemetry::DataPoint {
        timestamp: now, // ← Should be seconds, not milliseconds
        value: MetricValue::Float(99.5),
        quality: Some(1.0),
    };

    storage
        .write(device_id, "virtual.avg", data_point)
        .await
        .expect("Failed to write virtual metric");

    // Query the virtual metric
    let start = now - 60;
    let end = now + 60;

    let points = storage
        .query(device_id, "virtual.avg", start, end)
        .await
        .expect("Failed to query virtual metric");

    assert_eq!(points.len(), 1, "Should find virtual metric");
    assert_eq!(points[0].timestamp, now, "Timestamp should match");
    assert_eq!(
        points[0].value,
        MetricValue::Float(99.5),
        "Value should match"
    );
}
