//! Core system services state.
//!
//! Contains fundamental services used across the application:
//! - EventBus for event-driven communication
//! - MessageManager for unified messaging
//!
//! Note: ExtensionRegistry has been moved to ExtensionState for proper decoupling.

use std::sync::Arc;

use heramind_core::EventBus;
use heramind_messages::MessageManager;

/// Core system services state.
///
/// Provides access to fundamental cross-cutting services.
#[derive(Clone)]
pub struct CoreState {
    /// Event bus for system-wide event distribution.
    pub event_bus: Option<Arc<EventBus>>,

    /// Message manager for unified messages/notifications system.
    pub message_manager: Arc<MessageManager>,
}

impl CoreState {
    /// Create a new core state.
    pub fn new(event_bus: Option<Arc<EventBus>>, message_manager: Arc<MessageManager>) -> Self {
        Self {
            event_bus,
            message_manager,
        }
    }

    /// Create a minimal core state (for testing).
    #[cfg(test)]
    pub fn minimal() -> Self {
        use heramind_core::EventBus;
        Self {
            event_bus: Some(Arc::new(EventBus::new())),
            message_manager: Arc::new(MessageManager::new()),
        }
    }
}
