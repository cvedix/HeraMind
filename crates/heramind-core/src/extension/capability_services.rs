//! Capability Services Container
//!
//! This module provides a simple container for holding service references
//! that can be used by capability providers. Uses `Any` for dynamic typing
//! to avoid circular dependencies between crates.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Container for capability services.
///
/// Uses `Arc<dyn Any>` to hold `Arc<T>` references without requiring
/// trait definitions. Services are downcast to their concrete types
/// when used by capability providers.
#[derive(Clone, Default)]
pub struct CapabilityServices {
    services: HashMap<&'static str, Arc<dyn Any + Send + Sync>>,
}

impl CapabilityServices {
    /// Create a new empty services container
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a service to the container
    pub fn with_service<T: 'static + Send + Sync>(
        mut self,
        key: &'static str,
        service: Arc<T>,
    ) -> Self {
        self.services.insert(key, Arc::new(service));
        self
    }

    /// Get a service from the container
    pub fn get<T: 'static>(&self, key: &'static str) -> Option<Arc<T>> {
        let inner: &Arc<T> = self.services.get(key)?.downcast_ref::<Arc<T>>()?;
        Some(inner.clone())
    }

    /// Check if a service exists
    pub fn has(&self, key: &'static str) -> bool {
        self.services.contains_key(key)
    }
}

/// Service keys for built-in capabilities
pub mod keys {
    /// Device service key
    pub const DEVICE_SERVICE: &str = "device_service";
    /// Telemetry storage key
    pub const TELEMETRY_STORAGE: &str = "telemetry_storage";
    /// Rule engine key
    pub const RULE_ENGINE: &str = "rule_engine";
    /// Extension registry key
    pub const EXTENSION_REGISTRY: &str = "extension_registry";
    /// Event bus key
    pub const EVENT_BUS: &str = "event_bus";
    /// Agent manager key
    pub const AGENT_MANAGER: &str = "agent_manager";
    /// Agent store key
    pub const AGENT_STORE: &str = "agent_store";
    /// Session manager key (for ChatStream capability)
    pub const SESSION_MANAGER: &str = "session_manager";
}
