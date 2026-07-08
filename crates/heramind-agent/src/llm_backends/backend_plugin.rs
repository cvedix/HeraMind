//! LLM backend plugin system for runtime extensibility.
//!
//! This module provides a plugin registry that allows dynamic registration
//! of LLM backends at runtime, enabling third-party extensions without
//! modifying the core codebase.

use serde_json::Value;
use std::sync::OnceLock;

#[cfg(not(feature = "cloud"))]
use std::marker::PhantomData;

#[cfg(feature = "cloud")]
use std::collections::HashMap;

#[cfg(feature = "cloud")]
use std::sync::RwLock;

#[cfg(feature = "cloud")]
use std::sync::Arc;

#[cfg(feature = "cloud")]
use heramind_core::llm::backend::{LlmError, LlmRuntime};

/// Plugin trait for LLM backend implementations.
pub trait LlmBackendPlugin: Send + Sync {
    /// Get the plugin identifier (e.g., "ollama", "openai")
    fn backend_id(&self) -> &'static str;

    /// Get the plugin display name
    fn display_name(&self) -> &'static str;

    /// Create a runtime instance from configuration
    #[cfg(feature = "cloud")]
    fn create_runtime(&self, config: &Value) -> Result<Box<dyn LlmRuntime>, LlmError>;
}

/// Dynamic backend plugin from a factory function.
pub struct DynBackendPlugin {
    id: &'static str,
    name: &'static str,
    #[cfg(feature = "cloud")]
    #[allow(clippy::type_complexity)]
    factory: Box<dyn Fn(&Value) -> Result<Box<dyn LlmRuntime>, LlmError> + Send + Sync>,
    #[cfg(not(feature = "cloud"))]
    _marker: PhantomData<()>,
}

impl DynBackendPlugin {
    #[cfg(feature = "cloud")]
    pub fn new(
        id: &'static str,
        name: &'static str,
        factory: impl Fn(&Value) -> Result<Box<dyn LlmRuntime>, LlmError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            id,
            name,
            factory: Box::new(factory),
        }
    }

    #[cfg(not(feature = "cloud"))]
    pub fn new(
        id: &'static str,
        name: &'static str,
        _factory: impl Fn(&Value) -> Result<(), ()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            id,
            name,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "cloud")]
impl LlmBackendPlugin for DynBackendPlugin {
    fn backend_id(&self) -> &'static str {
        self.id
    }

    fn display_name(&self) -> &'static str {
        self.name
    }

    fn create_runtime(&self, config: &Value) -> Result<Box<dyn LlmRuntime>, LlmError> {
        (self.factory)(config)
    }
}

#[cfg(not(feature = "cloud"))]
impl LlmBackendPlugin for DynBackendPlugin {
    fn backend_id(&self) -> &'static str {
        self.id
    }

    fn display_name(&self) -> &'static str {
        self.name
    }
}

/// Global backend registry.
///
/// When the `cloud` feature is enabled, this registry stores plugin backends.
/// When disabled, it's a placeholder that compiles but does nothing.
pub struct BackendRegistry {
    #[cfg(feature = "cloud")]
    backends: RwLock<HashMap<String, Arc<dyn LlmBackendPlugin>>>,
    #[cfg(not(feature = "cloud"))]
    _marker: PhantomData<()>,
}

impl BackendRegistry {
    /// Get the global registry instance.
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<BackendRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| Self {
            #[cfg(feature = "cloud")]
            backends: RwLock::new(HashMap::new()),
            #[cfg(not(feature = "cloud"))]
            _marker: PhantomData,
        })
    }

    /// Register a backend plugin.
    #[cfg(feature = "cloud")]
    pub fn register(&self, plugin: Arc<dyn LlmBackendPlugin>) {
        let mut backends = self.backends.write().unwrap_or_else(|e| {
            tracing::error!("Failed to acquire write lock on backend registry: {}", e);
            e.into_inner()
        });
        backends.insert(plugin.backend_id().to_string(), plugin);
    }

    /// Get a registered plugin by ID.
    #[cfg(feature = "cloud")]
    pub fn get(&self, id: &str) -> Option<Arc<dyn LlmBackendPlugin>> {
        let backends = self.backends.read().unwrap_or_else(|e| {
            tracing::error!("Failed to acquire read lock on backend registry: {}", e);
            e.into_inner()
        });
        backends.get(id).cloned()
    }

    /// List all registered backend IDs.
    #[cfg(feature = "cloud")]
    pub fn list(&self) -> Vec<String> {
        let backends = self.backends.read().unwrap_or_else(|e| {
            tracing::error!("Failed to acquire read lock on backend registry: {}", e);
            e.into_inner()
        });
        backends.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_singleton() {
        let r1 = BackendRegistry::global();
        let r2 = BackendRegistry::global();
        // Same pointer
        assert!(std::ptr::eq(r1, r2));
    }
}
