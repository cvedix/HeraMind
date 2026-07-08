//! FFI utilities for Native extensions
//!
//! This module provides FFI-related utilities and helpers.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Convert a C string pointer to a Rust string
///
/// # Safety
///
/// The pointer must be valid and point to a null-terminated string.
pub unsafe fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
}

/// Convert a Rust string to a C string pointer
///
/// # Safety
///
/// The returned pointer is valid only as long as the returned CString is alive.
/// This function returns the CString along with the pointer to ensure proper lifetime management.
///
/// # Example
/// ```ignore
/// let (cstr, ptr) = string_to_c_str_owned("hello");
/// // ptr is valid while cstr is in scope
/// unsafe { /* use ptr */ }
/// // cstr dropped here, ptr becomes invalid
/// ```
pub fn string_to_c_str_owned(s: &str) -> Option<(CString, *const c_char)> {
    CString::new(s).ok().map(|cstr| {
        let ptr = cstr.as_ptr();
        (cstr, ptr)
    })
}

/// Safe wrapper for FFI calls that may panic
///
/// This function catches panics in FFI calls and converts them to errors.
pub fn safe_ffi_call<T, E, F>(fn_name: &str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E> + std::panic::UnwindSafe,
    E: From<String>,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => result,
        Err(panic_payload) => {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic in extension FFI".to_string()
            };
            Err(E::from(format!(
                "Extension panicked in {}: {}",
                fn_name, msg
            )))
        }
    }
}

/// Create a C-compatible metadata structure
#[macro_export]
macro_rules! create_c_metadata {
    ($id:literal, $name:literal, $version:literal, $metric_count:expr, $command_count:expr) => {{
        use std::ffi::CStr;

        let id = CStr::from_bytes_with_nul(concat!($id, "\0").as_bytes()).unwrap();
        let name = CStr::from_bytes_with_nul(concat!($name, "\0").as_bytes()).unwrap();
        let version = CStr::from_bytes_with_nul(concat!($version, "\0").as_bytes()).unwrap();

        $crate::CExtensionMetadata {
            abi_version: $crate::SDK_ABI_VERSION,
            id: id.as_ptr(),
            name: name.as_ptr(),
            version: version.as_ptr(),
            description: std::ptr::null(),
            author: std::ptr::null(),
            metric_count: $metric_count,
            command_count: $command_count,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_str_conversion() {
        let rust_str = "hello";
        // Use the owned version that properly manages CString lifetime
        let (cstr, c_ptr) = string_to_c_str_owned(rust_str).expect("Failed to create CString");
        unsafe {
            let converted = c_str_to_string(c_ptr);
            assert_eq!(converted, Some(rust_str.to_string()));
        }
        // cstr is dropped here, keeping it alive during the unsafe block
        let _ = cstr;
    }

    #[test]
    fn test_c_str_to_string_null() {
        // Test null pointer handling
        let result = unsafe { c_str_to_string(std::ptr::null()) };
        assert_eq!(result, None);
    }

    #[test]
    fn test_c_str_to_string_empty() {
        let (cstr, c_ptr) = string_to_c_str_owned("").expect("Failed to create empty CString");
        unsafe {
            let converted = c_str_to_string(c_ptr);
            assert_eq!(converted, Some("".to_string()));
        }
        let _ = cstr;
    }

    #[test]
    fn test_string_to_c_str_owned_special_chars() {
        // Test with special characters that should work
        let test_str = "hello world!";
        let (cstr, c_ptr) = string_to_c_str_owned(test_str).expect("Failed to create CString");
        unsafe {
            let converted = c_str_to_string(c_ptr);
            assert_eq!(converted, Some(test_str.to_string()));
        }
        let _ = cstr;
    }

    #[test]
    fn test_string_to_c_str_owned_null_byte() {
        // CString::new fails on strings containing null bytes
        let test_str = "hello\0world";
        let result = string_to_c_str_owned(test_str);
        assert!(result.is_none(), "Should fail on string with null byte");
    }

    #[test]
    fn test_safe_ffi_call_success() {
        let result: Result<i32, String> = safe_ffi_call("test_fn", || Ok(42));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_safe_ffi_call_error() {
        let result: Result<i32, String> =
            safe_ffi_call("test_fn", || Err("test error".to_string()));
        assert_eq!(result, Err("test error".to_string()));
    }

    #[test]
    fn test_safe_ffi_call_panic_string() {
        let result: Result<i32, String> = safe_ffi_call("test_fn", || {
            panic!("test panic message");
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("test panic message"));
        assert!(err.contains("test_fn"));
    }

    #[test]
    fn test_safe_ffi_call_panic_str() {
        let result: Result<i32, String> = safe_ffi_call("test_fn", || {
            panic!("static str panic");
        });
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("static str panic"));
    }

    #[test]
    fn test_safe_ffi_call_panic_unknown() {
        // Test with a panic that downcasts to neither &str nor String
        let result: Result<i32, String> = safe_ffi_call("test_fn", || std::panic::panic_any(42i32));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown panic"));
    }
}
