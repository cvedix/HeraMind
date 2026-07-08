//! Global event state for event subscription management
//!
//! Manages event subscriptions and queues for the extension runner.

use std::collections::HashMap;

use serde_json::json;
use tracing::warn;

/// Maximum number of events per subscriber queue before dropping the oldest.
const MAX_QUEUE_SIZE: usize = 1000;

/// Global event state shared across the runner
pub(crate) struct GlobalEventState {
    subscriptions: parking_lot::RwLock<HashMap<i64, String>>,
    queues: parking_lot::RwLock<HashMap<i64, Vec<serde_json::Value>>>,
    next_id: std::sync::atomic::AtomicI64,
}

impl GlobalEventState {
    pub(crate) fn new() -> Self {
        Self {
            subscriptions: parking_lot::RwLock::new(HashMap::new()),
            queues: parking_lot::RwLock::new(HashMap::new()),
            next_id: std::sync::atomic::AtomicI64::new(1),
        }
    }

    pub(crate) fn subscribe(&self, event_type: String) -> i64 {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.subscriptions.write().insert(id, event_type);
        id
    }

    pub(crate) fn unsubscribe(&self, id: i64) -> bool {
        self.queues.write().remove(&id);
        self.subscriptions.write().remove(&id).is_some()
    }

    pub(crate) fn push_event(&self, event_type: &str, payload: serde_json::Value) {
        let subscriptions = self.subscriptions.read();
        let mut queues = self.queues.write();

        for (id, sub_type) in subscriptions.iter() {
            if sub_type == "all"
                || sub_type == event_type
                || event_type.starts_with(&format!("{}::", sub_type))
            {
                let event = json!({
                    "event_type": event_type,
                    "payload": payload,
                });
                let queue = queues.entry(*id).or_default();
                if queue.len() >= MAX_QUEUE_SIZE {
                    queue.remove(0);
                    warn!(
                        subscriber_id = *id,
                        max = MAX_QUEUE_SIZE,
                        "Event queue full, dropped oldest event"
                    );
                }
                queue.push(event);
            }
        }
    }

    pub(crate) fn take_events(&self, id: i64) -> Vec<serde_json::Value> {
        self.queues.write().remove(&id).unwrap_or_default()
    }
}

static GLOBAL_EVENT_STATE: std::sync::OnceLock<GlobalEventState> = std::sync::OnceLock::new();

pub(crate) fn get_global_event_state() -> &'static GlobalEventState {
    GLOBAL_EVENT_STATE.get_or_init(GlobalEventState::new)
}
