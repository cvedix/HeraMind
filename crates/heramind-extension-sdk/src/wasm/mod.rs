//! WASM-specific extension utilities
//!
//! This module provides utilities for WASM extensions with a unified
//! capability invocation interface.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Developer API                              │
//! │    device::get_metrics(&ctx, "device-1")                    │
//! │    event::publish(&ctx, "device_changed", payload)          │
//! └───────────────────────────┬─────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              ExtensionContext.invoke_capability()           │
//! │    Unified entry point for all capability calls             │
//! └───────────────────────────┬─────────────────────────────────┘
//!                             │
//!            ┌────────────────┴────────────────┐
//!            ▼                                 ▼
//! ┌──────────────────────┐         ┌──────────────────────┐
//! │    Native Context    │         │    WASM Context      │
//! │    (async, direct)   │         │    (sync, FFI)       │
//! └──────────────────────┘         └──────────────────────┘
//!                                             │
//!                                             ▼
//!                                  ┌──────────────────────┐
//!                                  │  host_invoke_        │
//!                                  │  capability()        │
//!                                  │  (Single FFI entry)  │
//!                                  └──────────────────────┘
//! ```
//!
//! # Memory Layout
//!
//! ```text
//! WASM Linear Memory:
//! ┌─────────────────────────────────────────────────────────┐
//! │ 0x0000 - 0xFFFF  │ Stack / Heap (managed by WASM runtime) │
//! ├─────────────────────────────────────────────────────────┤
//! │ 0x10000 (64KB)   │ Result Buffer Start                   │
//! │                  │ Used for returning JSON data to host   │
//! │ 0x1FFFF (128KB)  │ Result Buffer End (64KB max)          │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod bindings;
pub mod context;
pub mod types;

// Re-export main types
pub use bindings::{invoke_capability_raw, log, timestamp_ms};
pub use context::{capabilities, EventSubscription, ExtensionContext};
pub use types::*;

/// Result buffer offset for WASM memory layout (64KB)
pub const RESULT_OFFSET: usize = 65536;

/// Maximum result size for WASM (64KB)
pub const RESULT_MAX_LEN: usize = 65536;

/// Input buffer offset for WASM memory layout (128KB)
pub const INPUT_OFFSET: usize = 131072;

/// Maximum input size for WASM (64KB)
pub const INPUT_MAX_LEN: usize = 65536;

/// Write a result string to WASM memory at the result offset
pub fn write_result(result: &str) -> i32 {
    let bytes = result.as_bytes();
    let write_len = bytes.len().min(RESULT_MAX_LEN - 1);

    unsafe {
        let dest = RESULT_OFFSET as *mut u8;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), dest, write_len);
        *dest.add(write_len) = 0; // Null terminator
    }

    write_len as i32
}

/// Write bytes to WASM memory at a specific offset
pub fn write_bytes_at_offset(offset: usize, data: &[u8]) -> i32 {
    let write_len = data.len().min(RESULT_MAX_LEN - 1);

    unsafe {
        let dest = offset as *mut u8;
        core::ptr::copy_nonoverlapping(data.as_ptr(), dest, write_len);
        *dest.add(write_len) = 0; // Null terminator
    }

    write_len as i32
}

/// Read bytes from WASM memory
pub unsafe fn read_bytes_from_memory(ptr: i32, len: i32) -> Vec<u8> {
    if ptr == 0 || len <= 0 {
        return Vec::new();
    }
    let slice = core::slice::from_raw_parts(ptr as *const u8, len as usize);
    slice.to_vec()
}

/// Read a string from WASM memory
pub unsafe fn read_string_from_memory(ptr: i32, len: i32) -> String {
    let bytes = read_bytes_from_memory(ptr, len);
    String::from_utf8_lossy(&bytes).to_string()
}

/// Read JSON from WASM memory at the result offset
pub fn read_result_json() -> Option<serde_json::Value> {
    unsafe {
        // Find null terminator
        let start = RESULT_OFFSET as *const u8;
        let mut end = 0usize;
        while end < RESULT_MAX_LEN {
            if *start.add(end) == 0 {
                break;
            }
            end += 1;
        }

        if end == 0 {
            return None;
        }

        let slice = core::slice::from_raw_parts(start, end);
        let json_str = core::str::from_utf8(slice).ok()?;
        serde_json::from_str(json_str).ok()
    }
}

/// Parse JSON from bytes
pub fn parse_json(bytes: &[u8]) -> Result<serde_json::Value, String> {
    let json_str = core::str::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {}", e))
}

/// Create an error JSON response
pub fn error_response(error: &str) -> String {
    serde_json::to_string(&serde_json::json!({
        "success": false,
        "error": error
    }))
    .unwrap_or_else(|_| r#"{"success":false,"error":"JSON error"}"#.to_string())
}

/// Create a success JSON response
pub fn success_response(result: serde_json::Value) -> String {
    serde_json::to_string(&serde_json::json!({
        "success": true,
        "result": result
    }))
    .unwrap_or_else(|_| r#"{"success":false,"error":"JSON error"}"#.to_string())
}

/// Encode binary data as base64
pub fn encode_base64(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(data)
}

/// Get current timestamp in milliseconds (Unix epoch)
pub fn current_timestamp_ms() -> i64 {
    timestamp_ms()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_memory_layout_constants() {
        // Verify memory layout constants
        assert_eq!(RESULT_OFFSET, 65536); // 64KB
        assert_eq!(RESULT_MAX_LEN, 65536); // 64KB
        assert_eq!(INPUT_OFFSET, 131072); // 128KB
        assert_eq!(INPUT_MAX_LEN, 65536); // 64KB

        // Verify non-overlapping regions
        assert!(RESULT_OFFSET + RESULT_MAX_LEN <= INPUT_OFFSET);
    }

    #[test]
    fn test_write_result() {
        let result = r#"{"success":true,"data":42}"#;
        let len = write_result(result);

        assert_eq!(len as usize, result.len());
    }

    #[test]
    fn test_write_result_truncation() {
        // Create a string larger than max
        let large_string = "x".repeat(RESULT_MAX_LEN + 1000);
        let len = write_result(&large_string);

        // Should be truncated to max length - 1 (for null terminator)
        assert_eq!(len as usize, RESULT_MAX_LEN - 1);
    }

    #[test]
    fn test_write_bytes_at_offset() {
        let data = b"test data";
        let len = write_bytes_at_offset(RESULT_OFFSET, data);

        assert_eq!(len as usize, data.len());
    }

    #[test]
    fn test_read_bytes_from_memory_null_ptr() {
        unsafe {
            let result = read_bytes_from_memory(0, 10);
            assert!(result.is_empty());
        }
    }

    #[test]
    fn test_read_bytes_from_memory_invalid_len() {
        unsafe {
            let result = read_bytes_from_memory(100, -1);
            assert!(result.is_empty());

            let result = read_bytes_from_memory(100, 0);
            assert!(result.is_empty());
        }
    }

    #[test]
    fn test_read_string_from_memory_null_ptr() {
        unsafe {
            let result = read_string_from_memory(0, 10);
            assert!(result.is_empty());
        }
    }

    #[test]
    fn test_parse_json_valid() {
        let json_bytes = br#"{"key":"value"}"#;
        let result = parse_json(json_bytes);

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["key"], "value");
    }

    #[test]
    fn test_parse_json_invalid() {
        let invalid_bytes = b"not valid json";
        let result = parse_json(invalid_bytes);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_parse_json_invalid_utf8() {
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        let result = parse_json(invalid_utf8);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid UTF-8"));
    }

    #[test]
    fn test_error_response() {
        let error = error_response("Something went wrong");

        assert!(error.contains("success"));
        assert!(error.contains("false"));
        assert!(error.contains("Something went wrong"));

        let parsed: serde_json::Value = serde_json::from_str(&error).unwrap();
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"], "Something went wrong");
    }

    #[test]
    fn test_success_response() {
        let result = json!({"value": 42});
        let response = success_response(result.clone());

        assert!(response.contains("success"));
        assert!(response.contains("true"));

        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["result"]["value"], 42);
    }

    #[test]
    fn test_encode_base64() {
        let data = b"hello world";
        let encoded = encode_base64(data);

        // Verify base64 encoding
        assert!(!encoded.is_empty());
        assert_ne!(encoded, "hello world"); // Should be encoded, not plaintext

        // Verify it's valid base64
        use base64::{engine::general_purpose::STANDARD, Engine};
        let decoded = STANDARD.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_base64_empty() {
        let data = b"";
        let encoded = encode_base64(data);
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_encode_base64_binary() {
        let data: &[u8] = &[0x00, 0xFF, 0x80, 0x7F];
        let encoded = encode_base64(data);

        use base64::{engine::general_purpose::STANDARD, Engine};
        let decoded = STANDARD.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_current_timestamp_ms_returns_value() {
        // Note: This test just verifies the function returns a reasonable value
        // The actual timestamp_ms() calls the host function which may not be available in tests
        // So we just verify the function exists and compiles
        let _ = current_timestamp_ms();
    }

    #[test]
    fn test_result_offset_alignment() {
        // Verify result offset is properly aligned
        assert_eq!(
            RESULT_OFFSET % 8,
            0,
            "Result offset should be 8-byte aligned"
        );
        assert_eq!(INPUT_OFFSET % 8, 0, "Input offset should be 8-byte aligned");
    }

    #[test]
    fn test_json_roundtrip_complex() {
        let complex = json!({
            "device": {
                "id": "device-1",
                "metrics": [
                    {"name": "temp", "value": 25.5},
                    {"name": "humidity", "value": 65.0},
                ],
            },
            "timestamp": 1700000000000i64,
            "valid": true,
            "notes": null,
        });

        // Test error_response with complex data
        let error_str = error_response(&serde_json::to_string(&complex).unwrap());
        assert!(error_str.contains("success"));

        // Test success_response with complex data
        let success_str = success_response(complex.clone());
        let parsed: serde_json::Value = serde_json::from_str(&success_str).unwrap();
        assert_eq!(parsed["result"]["device"]["id"], "device-1");
    }

    #[test]
    fn test_write_result_empty() {
        let len = write_result("");
        assert_eq!(len, 0);
    }

    #[test]
    fn test_write_result_unicode() {
        let unicode_str = "你好世界 🌍";
        let len = write_result(unicode_str);

        // Unicode characters have multi-byte UTF-8 encoding
        assert!(len as usize >= unicode_str.chars().count());
    }
}
