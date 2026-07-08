//! Core traits and types for HeraMind.
//!
//! This crate defines the foundational abstractions used across the project.

// alerts module removed - use heramind_messages instead
pub mod brand;
pub mod config;
pub mod dashboard;
pub mod datasource;
pub mod error;
pub mod event;
pub mod eventbus;
pub mod extension;
pub mod llm;
pub mod message;
pub mod tools;

pub use llm::LlmError;

// Exports
pub use llm::backend::{BackendCapabilities, GenerationParams, LlmRuntime};

pub use message::{Content, ContentPart, Message, MessageRole};

// Event exports
pub use event::{MetricValue, HeraMindEvent};

// Event bus exports
pub use eventbus::EventBus;
