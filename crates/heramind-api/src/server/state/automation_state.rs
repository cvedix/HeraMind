//! Automation and rules state.
//!
//! Contains all automation-related services:
//! - RuleEngine for DSL rule evaluation
//! - RuleStore for persistent rule storage
//! - AutomationStore for unified automations
//! - TransformEngine for data processing

use std::sync::Arc;

use crate::automation::{store::SharedAutomationStore, transform::TransformEngine};
use heramind_rules::{store::RuleStore, RuleEngine, UnifiedValueProvider};

/// Automation and rules state.
///
/// Provides access to rule engine, automation stores, and related services.
#[derive(Clone)]
pub struct AutomationState {
    /// Rule engine for DSL rule evaluation.
    pub rule_engine: Arc<RuleEngine>,

    /// Typed value provider for direct cache updates.
    pub value_provider: Arc<UnifiedValueProvider>,

    /// Rule store for persistent rule storage.
    pub rule_store: Option<Arc<RuleStore>>,

    /// Automation store for unified automations (rules + transforms).
    pub automation_store: Option<Arc<SharedAutomationStore>>,

    /// Transform engine for data processing.
    pub transform_engine: Option<Arc<TransformEngine>>,
}

impl AutomationState {
    /// Create a new automation state.
    pub fn new(
        value_provider: Arc<UnifiedValueProvider>,
        rule_engine: Arc<RuleEngine>,
        rule_store: Option<Arc<RuleStore>>,
        automation_store: Option<Arc<SharedAutomationStore>>,
        transform_engine: Option<Arc<TransformEngine>>,
    ) -> Self {
        // Set rule store in rule engine for persistent trigger count
        if let Some(ref store) = rule_store {
            rule_engine.set_rule_store(store.clone());
        }

        Self {
            rule_engine,
            value_provider,
            rule_store,
            automation_store,
            transform_engine,
        }
    }

    /// Create a minimal automation state (for testing).
    #[cfg(test)]
    pub fn minimal() -> Self {
        use heramind_rules::InMemoryValueProvider;
        let provider = Arc::new(InMemoryValueProvider::new());
        // Create a dummy UnifiedValueProvider for test (won't be used)
        let unified = Arc::new(UnifiedValueProvider::new());
        Self {
            rule_engine: Arc::new(RuleEngine::new(provider)),
            value_provider: unified,
            rule_store: None,
            automation_store: None,
            transform_engine: None,
        }
    }
}
