//! Error types for the tools crate.

// Re-export the core error type
pub use heramind_core::error::Error as HeraMindError;

/// Tool error types.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Tool is disabled (hidden from the LLM by the user via the Extensions
    /// page). Surfaced when a stale tool definition still reaches execute(),
    /// e.g. mid-session toggle on the chat path before refresh.
    #[error("Tool disabled: {0}")]
    Disabled(String),

    /// Invalid arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Canceled
    #[error("Operation canceled")]
    Canceled,

    /// Configuration error (Phase 3.2)
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Result type for tool operations.
pub type Result<T> = std::result::Result<T, ToolError>;

// Convert ToolError to HeraMindError
impl From<ToolError> for HeraMindError {
    fn from(e: ToolError) -> Self {
        match e {
            ToolError::NotFound(s) => HeraMindError::NotFound(s),
            ToolError::Disabled(s) => HeraMindError::Unauthorized(format!("Tool {s} is disabled")),
            ToolError::InvalidArguments(s) => HeraMindError::Validation(s),
            ToolError::Execution(s) => HeraMindError::Tool(s),
            ToolError::Serialization(s) => HeraMindError::Serialization(s),
            ToolError::PermissionDenied(s) => HeraMindError::Unauthorized(s),
            ToolError::Timeout => HeraMindError::Timeout("Tool operation timed out".to_string()),
            ToolError::Canceled => HeraMindError::Internal("Operation canceled".to_string()),
            ToolError::ConfigurationError(s) => HeraMindError::Internal(s),
        }
    }
}

// External error conversions
impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        ToolError::Serialization(err.to_string())
    }
}

impl From<heramind_storage::Error> for ToolError {
    fn from(err: heramind_storage::Error) -> Self {
        ToolError::Execution(err.to_string())
    }
}

// Convert from image_utils::ImageIoError
impl From<crate::image_utils::ImageIoError> for ToolError {
    fn from(err: crate::image_utils::ImageIoError) -> Self {
        match err {
            crate::image_utils::ImageIoError::InvalidArguments(s) => ToolError::InvalidArguments(s),
            crate::image_utils::ImageIoError::PermissionDenied(s) => ToolError::PermissionDenied(s),
            crate::image_utils::ImageIoError::Execution(s) => ToolError::Execution(s),
            crate::image_utils::ImageIoError::Timeout => ToolError::Timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ToolError::NotFound("test_tool".to_string());
        assert!(err.to_string().contains("test_tool"));
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let tool_err: ToolError = json_err.into();
        assert!(matches!(tool_err, ToolError::Serialization(_)));
    }

    #[test]
    fn test_tool_error_to_neo_talk_error() {
        let tool_err = ToolError::NotFound("my_tool".to_string());
        let neo_err: HeraMindError = tool_err.into();
        assert!(matches!(neo_err, HeraMindError::NotFound(_)));

        let args_err = ToolError::InvalidArguments("bad args".to_string());
        let neo_err: HeraMindError = args_err.into();
        assert!(matches!(neo_err, HeraMindError::Validation(_)));
    }
}
