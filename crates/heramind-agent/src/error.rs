//! Error types for the agent crate.
//!
//! This module re-exports the unified error type from core.

// Re-export the core error type
pub use heramind_core::error::Error as HeraMindError;
pub use heramind_core::error::Result as CoreResult;

/// Result type for agent operations.
pub type Result<T> = CoreResult<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HeraMindError::NotFound("test_key".to_string());
        assert!(err.to_string().contains("test_key"));
    }

    #[test]
    fn test_error_convenience_constructors() {
        let err = HeraMindError::not_found("test_key");
        assert!(err.to_string().contains("test_key"));

        let validation_err = HeraMindError::validation("invalid input");
        assert!(validation_err.to_string().contains("invalid input"));
    }

    #[test]
    fn test_timeout_error() {
        let err = HeraMindError::timeout("operation timed out");
        assert!(err.to_string().contains("timed out"));
    }
}
