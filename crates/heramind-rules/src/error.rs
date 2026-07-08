//! Error types for the rules crate.

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for rule operations
pub type Result<T> = std::result::Result<T, RuleError>;
