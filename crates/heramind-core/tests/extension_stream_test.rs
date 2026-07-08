//! Comprehensive Unit Tests for Extension Stream Module
//!
//! Tests cover:
//! - StreamDataType creation and MIME type handling
//! - DataChunk creation and serialization
//! - StreamResult creation and parsing
//! - StreamError handling
//! - StreamCapability builder pattern
//! - FlowControl settings
//! - StreamSession management
//! - SessionStats tracking

#![allow(clippy::assertions_on_constants)]

use heramind_core::extension::stream::{
    ClientInfo, DataChunk, FlowControl, SessionStats, StreamCapability, StreamDataType,
    StreamDirection, StreamError, StreamMode, StreamResult, StreamSession,
};
use serde_json::json;

// ============================================================================
// StreamDataType Tests
// ============================================================================

#[test]
fn test_stream_data_type_binary() {
    let dt = StreamDataType::Binary;
    assert_eq!(dt.mime_type(), "application/octet-stream");
}

#[test]
fn test_stream_data_type_text() {
    let dt = StreamDataType::Text;
    assert_eq!(dt.mime_type(), "text/plain");
}

#[test]
fn test_stream_data_type_json() {
    let dt = StreamDataType::Json;
    assert_eq!(dt.mime_type(), "application/json");
}

#[test]
fn test_stream_data_type_image() {
    let formats = vec![
        ("jpeg", "image/jpeg"),
        ("jpg", "image/jpeg"),
        ("png", "image/png"),
        ("gif", "image/gif"),
        ("webp", "image/webp"),
        ("bmp", "image/bmp"),
        ("tiff", "image/tiff"),
    ];

    for (format, expected_mime) in formats {
        let dt = StreamDataType::Image {
            format: format.to_string(),
        };
        assert_eq!(dt.mime_type(), expected_mime);
    }
}

#[test]
fn test_stream_data_type_audio() {
    let formats = vec![
        ("pcm", "audio/pcm"),
        ("mp3", "audio/mpeg"),
        ("aac", "audio/aac"),
        ("wav", "audio/wav"),
        ("ogg", "audio/ogg"),
        ("flac", "audio/flac"),
    ];

    for (format, expected_mime) in formats {
        let dt = StreamDataType::Audio {
            format: format.to_string(),
            sample_rate: 48000,
            channels: 2,
        };
        assert_eq!(dt.mime_type(), expected_mime);
    }
}

#[test]
fn test_stream_data_type_video() {
    let codecs = vec![
        ("h264", "video/h264"),
        ("h.264", "video/h264"),
        ("h265", "video/h265"),
        ("h.265", "video/h265"),
        ("hevc", "video/h265"),
        ("vp8", "video/vp8"),
        ("vp9", "video/vp9"),
        ("av1", "video/av1"),
    ];

    for (codec, expected_mime) in codecs {
        let dt = StreamDataType::Video {
            codec: codec.to_string(),
            width: 1920,
            height: 1080,
            fps: 30,
        };
        assert_eq!(dt.mime_type(), expected_mime);
    }
}

#[test]
fn test_stream_data_type_sensor() {
    let dt = StreamDataType::Sensor {
        sensor_type: "temperature".to_string(),
    };
    assert_eq!(dt.mime_type(), "application/x-sensor.temperature");

    let dt = StreamDataType::Sensor {
        sensor_type: "humidity".to_string(),
    };
    assert_eq!(dt.mime_type(), "application/x-sensor.humidity");
}

#[test]
fn test_stream_data_type_custom() {
    let dt = StreamDataType::Custom {
        mime_type: "application/x-custom".to_string(),
    };
    assert_eq!(dt.mime_type(), "application/x-custom");
}

#[test]
fn test_stream_data_type_from_mime_binary() {
    let dt = StreamDataType::from_mime_type("application/octet-stream");
    assert_eq!(dt, Some(StreamDataType::Binary));
}

#[test]
fn test_stream_data_type_from_mime_text() {
    let dt = StreamDataType::from_mime_type("text/plain");
    assert_eq!(dt, Some(StreamDataType::Text));
}

#[test]
fn test_stream_data_type_from_mime_json() {
    let dt = StreamDataType::from_mime_type("application/json");
    assert_eq!(dt, Some(StreamDataType::Json));
}

#[test]
fn test_stream_data_type_from_mime_image() {
    let dt = StreamDataType::from_mime_type("image/png");
    assert!(matches!(dt, Some(StreamDataType::Image { format }) if format == "png"));

    let dt = StreamDataType::from_mime_type("image/jpeg");
    assert!(matches!(dt, Some(StreamDataType::Image { format }) if format == "jpeg"));
}

#[test]
fn test_stream_data_type_from_mime_audio() {
    let dt = StreamDataType::from_mime_type("audio/mpeg");
    assert!(matches!(dt, Some(StreamDataType::Audio { format, .. }) if format == "mpeg"));
}

#[test]
fn test_stream_data_type_from_mime_video() {
    let dt = StreamDataType::from_mime_type("video/h264");
    assert!(matches!(dt, Some(StreamDataType::Video { codec, .. }) if codec == "h264"));
}

#[test]
fn test_stream_data_type_from_mime_sensor() {
    let dt = StreamDataType::from_mime_type("application/x-sensor.pressure");
    assert!(
        matches!(dt, Some(StreamDataType::Sensor { sensor_type }) if sensor_type == "pressure")
    );
}

#[test]
fn test_stream_data_type_from_mime_custom() {
    let dt = StreamDataType::from_mime_type("application/x-unknown");
    assert!(
        matches!(dt, Some(StreamDataType::Custom { mime_type }) if mime_type == "application/x-unknown")
    );
}

#[test]
fn test_stream_data_type_serialization() {
    let dt = StreamDataType::Binary;
    let json = serde_json::to_string(&dt).unwrap();
    // Binary serializes as a simple string
    assert!(json.contains("binary"));

    let dt = StreamDataType::Image {
        format: "png".to_string(),
    };
    let json = serde_json::to_string(&dt).unwrap();
    assert!(json.contains("image"));
    assert!(json.contains("png"));
}

#[test]
fn test_stream_data_type_deserialization() {
    let json = r#""binary""#;
    let dt: StreamDataType = serde_json::from_str(json).unwrap();
    assert_eq!(dt, StreamDataType::Binary);

    let json = r#"{"image":{"format":"jpeg"}}"#;
    let dt: StreamDataType = serde_json::from_str(json).unwrap();
    assert!(matches!(dt, StreamDataType::Image { format } if format == "jpeg"));
}

// ============================================================================
// DataChunk Tests
// ============================================================================

#[test]
fn test_data_chunk_binary() {
    let chunk = DataChunk::binary(1, vec![0x01, 0x02, 0x03]);

    assert_eq!(chunk.sequence, 1);
    assert_eq!(chunk.data, vec![0x01, 0x02, 0x03]);
    assert_eq!(chunk.data_type, StreamDataType::Binary);
    assert!(!chunk.is_last);
    assert!(chunk.metadata.is_none());
}

#[test]
fn test_data_chunk_text() {
    let chunk = DataChunk::text(2, "Hello, World!".to_string());

    assert_eq!(chunk.sequence, 2);
    assert_eq!(chunk.data, "Hello, World!".as_bytes());
    assert_eq!(chunk.data_type, StreamDataType::Text);
}

#[test]
fn test_data_chunk_json() {
    let value = json!({"key": "value", "number": 42});
    let chunk = DataChunk::json(3, value.clone()).unwrap();

    assert_eq!(chunk.sequence, 3);
    assert_eq!(chunk.data_type, StreamDataType::Json);

    let parsed: serde_json::Value = serde_json::from_slice(&chunk.data).unwrap();
    assert_eq!(parsed, value);
}

#[test]
fn test_data_chunk_image() {
    let image_data = vec![0xFF, 0xD8, 0xFF]; // JPEG header
    let chunk = DataChunk::image(4, image_data.clone(), "jpeg".to_string());

    assert_eq!(chunk.sequence, 4);
    assert_eq!(chunk.data, image_data);
    assert!(matches!(chunk.data_type, StreamDataType::Image { format } if format == "jpeg"));
}

#[test]
fn test_data_chunk_with_last() {
    let chunk = DataChunk::binary(5, vec![]).with_last();

    assert!(chunk.is_last);
}

#[test]
fn test_data_chunk_with_metadata() {
    let metadata = json!({"source": "camera", "fps": 30});
    let chunk = DataChunk::binary(6, vec![]).with_metadata(metadata.clone());

    assert_eq!(chunk.metadata, Some(metadata));
}

#[test]
fn test_data_chunk_serialization() {
    let chunk = DataChunk::binary(7, vec![0x01, 0x02])
        .with_last()
        .with_metadata(json!({"test": true}));

    let json = serde_json::to_string(&chunk).unwrap();
    let parsed: DataChunk = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.sequence, 7);
    assert_eq!(parsed.data, vec![0x01, 0x02]);
    assert!(parsed.is_last);
    assert!(parsed.metadata.is_some());
}

// ============================================================================
// StreamResult Tests
// ============================================================================

#[test]
fn test_stream_result_success() {
    let result = StreamResult::success(
        Some(1),
        2,
        vec![0x01, 0x02, 0x03],
        StreamDataType::Binary,
        15.5,
    );

    assert_eq!(result.input_sequence, Some(1));
    assert_eq!(result.output_sequence, 2);
    assert_eq!(result.data, vec![0x01, 0x02, 0x03]);
    assert_eq!(result.data_type, StreamDataType::Binary);
    assert_eq!(result.processing_ms, 15.5);
    assert!(result.error.is_none());
}

#[test]
fn test_stream_result_json() {
    let value = json!({"result": "success", "count": 100});
    let result = StreamResult::json(Some(1), 2, value.clone(), 25.0).unwrap();

    assert_eq!(result.input_sequence, Some(1));
    assert_eq!(result.output_sequence, 2);
    assert_eq!(result.data_type, StreamDataType::Json);
    assert_eq!(result.processing_ms, 25.0);

    let parsed = result.as_json().unwrap();
    assert_eq!(parsed, value);
}

#[test]
fn test_stream_result_error() {
    let error = StreamError::fatal("PROCESSING_ERROR", "Failed to process data");
    let result = StreamResult::error(Some(1), error.clone());

    assert_eq!(result.input_sequence, Some(1));
    assert!(result.data.is_empty());
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap().code, "PROCESSING_ERROR");
}

#[test]
fn test_stream_result_with_metadata() {
    let metadata = json!({"model": "yolo-v8", "confidence": 0.95});
    let result = StreamResult::success(Some(1), 2, vec![], StreamDataType::Binary, 10.0)
        .with_metadata(metadata.clone());

    assert_eq!(result.metadata, Some(metadata));
}

#[test]
fn test_stream_result_as_json() {
    let value = json!({"detections": 5});
    let result = StreamResult::json(None, 1, value.clone(), 5.0).unwrap();

    let parsed = result.as_json().unwrap();
    assert_eq!(parsed, value);
}

#[test]
fn test_stream_result_as_text() {
    let result = StreamResult::success(
        Some(1),
        2,
        "Hello".as_bytes().to_vec(),
        StreamDataType::Text,
        2.0,
    );

    let text = result.as_text().unwrap();
    assert_eq!(text, "Hello");
}

#[test]
fn test_stream_result_as_json_wrong_type() {
    let result = StreamResult::success(Some(1), 2, vec![0x01, 0x02], StreamDataType::Binary, 1.0);

    // Should return Err for non-JSON type
    assert!(result.as_json().is_err());
}

#[test]
fn test_stream_result_serialization() {
    let result = StreamResult::success(Some(1), 2, vec![0x01, 0x02], StreamDataType::Binary, 10.0);

    let json = serde_json::to_string(&result).unwrap();
    let parsed: StreamResult = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.input_sequence, Some(1));
    assert_eq!(parsed.output_sequence, 2);
    assert_eq!(parsed.processing_ms, 10.0);
}

// ============================================================================
// StreamError Tests
// ============================================================================

#[test]
fn test_stream_error_new() {
    let err = StreamError::new("ERR001", "Something went wrong", true);

    assert_eq!(err.code, "ERR001");
    assert_eq!(err.message, "Something went wrong");
    assert!(err.retryable);
}

#[test]
fn test_stream_error_fatal() {
    let err = StreamError::fatal("FATAL001", "Critical error occurred");

    assert_eq!(err.code, "FATAL001");
    assert_eq!(err.message, "Critical error occurred");
    assert!(!err.retryable);
}

#[test]
fn test_stream_error_retryable() {
    let err = StreamError::retryable("TEMP001", "Temporary error");

    assert_eq!(err.code, "TEMP001");
    assert_eq!(err.message, "Temporary error");
    assert!(err.retryable);
}

#[test]
fn test_stream_error_display() {
    let err = StreamError::fatal("TEST_ERROR", "Test message");
    let display = format!("{}", err);

    assert!(display.contains("TEST_ERROR"));
    assert!(display.contains("Test message"));
}

#[test]
fn test_stream_error_serialization() {
    let err = StreamError::retryable("ERR002", "Retry later");

    let json = serde_json::to_string(&err).unwrap();
    let parsed: StreamError = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.code, "ERR002");
    assert_eq!(parsed.message, "Retry later");
    assert!(parsed.retryable);
}

// ============================================================================
// StreamCapability Tests
// ============================================================================

#[test]
fn test_stream_capability_default() {
    let cap = StreamCapability::default();

    assert_eq!(cap.direction, StreamDirection::Upload);
    assert_eq!(cap.mode, StreamMode::Stateless);
    // #[derive(Default)] => usize fields default to 0
    assert_eq!(cap.max_chunk_size, 0);
    assert_eq!(cap.preferred_chunk_size, 0);
    assert_eq!(cap.max_concurrent_sessions, 0);
}

#[test]
fn test_stream_capability_upload() {
    let cap = StreamCapability::upload();

    assert_eq!(cap.direction, StreamDirection::Upload);
    assert_eq!(cap.mode, StreamMode::Stateless);
}

#[test]
fn test_stream_capability_download() {
    let cap = StreamCapability::download();

    assert_eq!(cap.direction, StreamDirection::Download);
    assert_eq!(cap.mode, StreamMode::Stateless);
}

#[test]
fn test_stream_capability_stateful() {
    let cap = StreamCapability::stateful();

    assert_eq!(cap.direction, StreamDirection::Bidirectional);
    assert_eq!(cap.mode, StreamMode::Stateful);
}

#[test]
fn test_stream_capability_push() {
    let cap = StreamCapability::push();

    assert_eq!(cap.direction, StreamDirection::Download);
    assert_eq!(cap.mode, StreamMode::Push);
}

#[test]
fn test_stream_capability_with_data_type() {
    let cap = StreamCapability::upload()
        .with_data_type(StreamDataType::Image {
            format: "jpeg".to_string(),
        })
        .with_data_type(StreamDataType::Image {
            format: "png".to_string(),
        })
        .with_data_type(StreamDataType::Video {
            codec: "h264".to_string(),
            width: 1920,
            height: 1080,
            fps: 30,
        });

    // upload() starts with 1 default Binary type, then 3 more added = 4
    assert_eq!(cap.supported_data_types.len(), 4);
}

#[test]
fn test_stream_capability_with_chunk_size() {
    let cap = StreamCapability::upload().with_chunk_size(1024, 10 * 1024);

    assert_eq!(cap.preferred_chunk_size, 1024);
    assert_eq!(cap.max_chunk_size, 10 * 1024);
}

#[test]
fn test_stream_capability_serialization() {
    let cap = StreamCapability::upload()
        .with_data_type(StreamDataType::Binary)
        .with_chunk_size(1024, 4096);

    let json = serde_json::to_string(&cap).unwrap();
    let parsed: StreamCapability = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.direction, StreamDirection::Upload);
    assert_eq!(parsed.preferred_chunk_size, 1024);
    assert_eq!(parsed.max_chunk_size, 4096);
}

// ============================================================================
// StreamDirection Tests
// ============================================================================

#[test]
fn test_stream_direction_serialization() {
    let directions = vec![
        (StreamDirection::Upload, "upload"),
        (StreamDirection::Download, "download"),
        (StreamDirection::Bidirectional, "bidirectional"),
    ];

    for (dir, expected) in directions {
        let json = serde_json::to_string(&dir).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));

        let parsed: StreamDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, dir);
    }
}

// ============================================================================
// StreamMode Tests
// ============================================================================

#[test]
fn test_stream_mode_serialization() {
    let modes = vec![
        (StreamMode::Stateless, "stateless"),
        (StreamMode::Stateful, "stateful"),
        (StreamMode::Push, "push"),
    ];

    for (mode, expected) in modes {
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));

        let parsed: StreamMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, mode);
    }
}

// ============================================================================
// FlowControl Tests
// ============================================================================

#[test]
fn test_flow_control_default() {
    let fc = FlowControl::default();

    // Default delegates to default_stream() which enables backpressure
    assert!(fc.supports_backpressure);
    assert_eq!(fc.window_size, 64 * 1024);
    assert!(!fc.supports_throttling);
    assert_eq!(fc.max_rate, 0);
}

#[test]
fn test_flow_control_serialization() {
    let fc = FlowControl {
        supports_backpressure: true,
        window_size: 20,
        supports_throttling: true,
        max_rate: 100,
    };

    let json = serde_json::to_string(&fc).unwrap();
    let parsed: FlowControl = serde_json::from_str(&json).unwrap();

    assert!(parsed.supports_backpressure);
    assert_eq!(parsed.window_size, 20);
    assert!(parsed.supports_throttling);
    assert_eq!(parsed.max_rate, 100);
}

// ============================================================================
// StreamSession Tests
// ============================================================================

#[test]
fn test_stream_session_new() {
    let client_info = ClientInfo {
        client_id: "client-1".to_string(),
        ip_addr: Some("192.168.1.1".to_string()),
        user_agent: Some("TestClient/1.0".to_string()),
    };

    let session = StreamSession::new(
        "session-123".to_string(),
        "ext-weather".to_string(),
        json!({"mode": "streaming"}),
        client_info.clone(),
    );

    assert_eq!(session.id, "session-123");
    assert_eq!(session.extension_id, "ext-weather");
    assert_eq!(session.client_info.as_ref().unwrap().client_id, "client-1");
    assert!(session.started_at > 0);
}

#[test]
fn test_stream_session_age() {
    let session = StreamSession::new(
        "session-456".to_string(),
        "ext-sensor".to_string(),
        json!({}),
        ClientInfo {
            client_id: "test".to_string(),
            ip_addr: None,
            user_agent: None,
        },
    );

    // Age should be very small (just created)
    assert!(session.age_ms() < 1000);
    assert!(session.age_secs() == 0);
}

#[test]
fn test_stream_session_serialization() {
    let session = StreamSession::new(
        "session-789".to_string(),
        "ext-video".to_string(),
        json!({"codec": "h264"}),
        ClientInfo {
            client_id: "client-2".to_string(),
            ip_addr: None,
            user_agent: None,
        },
    );

    // Session should be debug-printable
    let debug = format!("{:?}", session);
    assert!(debug.contains("session-789"));
    assert!(debug.contains("ext-video"));
}

// ============================================================================
// ClientInfo Tests
// ============================================================================

#[test]
fn test_client_info_serialization() {
    let info = ClientInfo {
        client_id: "client-abc".to_string(),
        ip_addr: Some("10.0.0.1".to_string()),
        user_agent: Some("HeraMind/2.0".to_string()),
    };

    let json = serde_json::to_string(&info).unwrap();
    let parsed: ClientInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.client_id, "client-abc");
    assert_eq!(parsed.ip_addr, Some("10.0.0.1".to_string()));
    assert_eq!(parsed.user_agent, Some("HeraMind/2.0".to_string()));
}

#[test]
fn test_client_info_minimal() {
    let info = ClientInfo {
        client_id: "minimal".to_string(),
        ip_addr: None,
        user_agent: None,
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("minimal"));
}

// ============================================================================
// SessionStats Tests
// ============================================================================

#[test]
fn test_session_stats_default() {
    let stats = SessionStats::default();

    assert_eq!(stats.input_chunks, 0);
    assert_eq!(stats.output_chunks, 0);
    assert_eq!(stats.input_bytes, 0);
    assert_eq!(stats.output_bytes, 0);
    assert_eq!(stats.errors, 0);
    assert!(stats.last_activity > 0);
}

#[test]
fn test_session_stats_record_input() {
    let mut stats = SessionStats::default();

    stats.record_input(100);
    stats.record_input(200);
    stats.record_input(300);

    assert_eq!(stats.input_chunks, 3);
    assert_eq!(stats.input_bytes, 600);
}

#[test]
fn test_session_stats_record_output() {
    let mut stats = SessionStats::default();

    stats.record_output(50);
    stats.record_output(150);

    assert_eq!(stats.output_chunks, 2);
    assert_eq!(stats.output_bytes, 200);
}

#[test]
fn test_session_stats_record_error() {
    let mut stats = SessionStats::default();

    stats.record_error();
    stats.record_error();
    stats.record_error();

    assert_eq!(stats.errors, 3);
}

#[test]
fn test_session_stats_last_activity() {
    let mut stats = SessionStats::default();
    let initial = stats.last_activity;

    // Wait long enough to cross a second boundary (timestamp is second-precision)
    std::thread::sleep(std::time::Duration::from_millis(1100));

    stats.record_input(100);

    assert!(stats.last_activity > initial);
}

#[test]
fn test_session_stats_serialization() {
    let mut stats = SessionStats::default();
    stats.record_input(1000);
    stats.record_output(500);
    stats.record_error();

    let json = serde_json::to_string(&stats).unwrap();
    let parsed: SessionStats = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.input_chunks, 1);
    assert_eq!(parsed.input_bytes, 1000);
    assert_eq!(parsed.output_chunks, 1);
    assert_eq!(parsed.output_bytes, 500);
    assert_eq!(parsed.errors, 1);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_chunk_to_result_flow() {
    // Create input chunk
    let input_chunk = DataChunk::json(1, json!({"image": "data"})).unwrap();

    // Simulate processing
    let output_result = StreamResult::json(
        Some(input_chunk.sequence),
        1,
        json!({"detections": [{"label": "person", "confidence": 0.95}]}),
        25.5,
    )
    .unwrap();

    assert_eq!(output_result.input_sequence, Some(1));
    assert_eq!(output_result.processing_ms, 25.5);

    let parsed = output_result.as_json().unwrap();
    assert!(parsed["detections"].is_array());
}

#[test]
fn test_capability_matching() {
    let cap = StreamCapability::upload()
        .with_data_type(StreamDataType::Image {
            format: "jpeg".to_string(),
        })
        .with_data_type(StreamDataType::Image {
            format: "png".to_string(),
        })
        .with_chunk_size(64 * 1024, 4 * 1024 * 1024);

    // Check if capability supports JPEG images
    let supports_jpeg = cap
        .supported_data_types
        .iter()
        .any(|dt| matches!(dt, StreamDataType::Image { format } if format == "jpeg"));
    assert!(supports_jpeg);

    // Check if capability supports PNG images
    let supports_png = cap
        .supported_data_types
        .iter()
        .any(|dt| matches!(dt, StreamDataType::Image { format } if format == "png"));
    assert!(supports_png);

    // Check chunk size limits
    assert!(cap.max_chunk_size >= cap.preferred_chunk_size);
}

#[test]
fn test_error_handling_flow() {
    // Create error
    let error = StreamError::retryable("TIMEOUT", "Processing timed out");

    // Create error result
    let result = StreamResult::error(Some(5), error);

    assert!(result.error.is_some());
    assert!(result.error.as_ref().unwrap().retryable);

    // Client can decide to retry based on error type
    if result.error.as_ref().map(|e| e.retryable).unwrap_or(false) {
        // Would retry
        assert!(true);
    }
}

#[test]
fn test_session_lifecycle() {
    // Create session
    let session = StreamSession::new(
        "session-lifecycle".to_string(),
        "ext-test".to_string(),
        json!({"mode": "stateful"}),
        ClientInfo {
            client_id: "test-client".to_string(),
            ip_addr: None,
            user_agent: None,
        },
    );

    // Track stats
    let mut stats = SessionStats::default();

    // Simulate streaming
    for i in 0..10 {
        let chunk = DataChunk::binary(i, vec![0u8; 1024]);
        stats.record_input(chunk.data.len() as u64);

        let result = StreamResult::success(
            Some(chunk.sequence),
            i,
            vec![0u8; 512],
            StreamDataType::Binary,
            5.0,
        );
        stats.record_output(result.data.len() as u64);
    }

    // Verify stats
    assert_eq!(stats.input_chunks, 10);
    assert_eq!(stats.input_bytes, 10 * 1024);
    assert_eq!(stats.output_chunks, 10);
    assert_eq!(stats.output_bytes, 10 * 512);

    // Session age should still be small
    assert!(session.age_secs() < 5);
}
