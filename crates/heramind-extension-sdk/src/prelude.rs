//! HeraMind Extension SDK Prelude
//!
//! Common imports for extension development.
//!
//! # Usage
//!
//! ```rust,ignore
//! use heramind_extension_sdk::prelude::*;
//! ```

// Re-export async_trait
pub use async_trait::async_trait;

// Re-export serde_json helpers
pub use serde_json::{json, Value};

// Core types from host module (Native target)
#[cfg(not(target_arch = "wasm32"))]
pub use crate::host::{
    AvailableCapabilities, CapabilityContext, CapabilityManifest, ClientInfo, DataChunk, Extension,
    ExtensionCapability, ExtensionContext, ExtensionContextConfig, FlowControl, SessionStats,
    StreamCapability, StreamDataType, StreamDirection, StreamError, StreamMode, StreamResult,
    StreamSession,
};

// IPC types (stable ABI)
pub use crate::ipc_types::{
    CExtensionMetadata, CommandDefinition, ExtensionCommand, ExtensionDescriptor, ExtensionError,
    ExtensionMetadata, ExtensionMetricValue, ExtensionRuntimeState, ExtensionStats, MetricDataType,
    MetricDescriptor, ParamMetricValue, ParameterDefinition, ParameterGroup, PushOutputMessage,
    Result, ValidationRule, ABI_VERSION,
};

// SDK helper types (for backward compatibility)
pub use crate::ipc_types::{
    ExtensionCommand as SdkCommandDefinition, ExtensionError as SdkExtensionError,
    ExtensionMetadata as SdkExtensionMetadata, MetricDataType as SdkMetricDataType,
    MetricDescriptor as SdkMetricDefinition, ParamMetricValue as SdkMetricValue,
    ParameterDefinition as SdkParameterDefinition, Result as SdkResult,
};

// Frontend types
pub use crate::FrontendManifest;

// Helper types
pub use crate::{CommandBuilder, MetricBuilder};

// Macros
pub use crate::{
    ext_debug, ext_error, ext_info, ext_log, ext_warn, metric_bool, metric_float, metric_int,
    metric_string, metric_value, heramind_export, static_commands, static_metadata, static_metrics,
};

// SDK constants
pub use crate::{MIN_HERAMIND_VERSION, SDK_ABI_VERSION, SDK_VERSION};

// Semver for version handling
pub use semver::Version;

// Re-export chrono for timestamp handling
#[cfg(not(target_arch = "wasm32"))]
pub use chrono;

// Tokio sync for Push mode
#[cfg(not(target_arch = "wasm32"))]
pub use tokio::sync::mpsc;
