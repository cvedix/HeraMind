//! Scheduler for push targets - event-driven and interval-based.

use crate::filter::DataSourceMatcher;
use crate::store::DataPushStore;
use crate::targets::create_destination;
use crate::template::TemplateRenderer;
use crate::types::*;
use anyhow::Result;
use heramind_core::EventBus;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handle for a running scheduled target.
struct ScheduledHandle {
    cancel: tokio::sync::watch::Sender<bool>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl ScheduledHandle {
    async fn stop(self) {
        let _ = self.cancel.send(true);
        let _ = self.join_handle.await;
    }
}

/// Helper to extract schedule info before moving target.
enum ScheduleInfo {
    EventDriven(Vec<String>),
    Interval(u64),
}

/// Manages running scheduled tasks for push targets.
pub struct PushScheduler {
    store: Arc<DataPushStore>,
    event_bus: Option<Arc<EventBus>>,
    renderer: Arc<TemplateRenderer>,
    handles: Arc<RwLock<HashMap<PushTargetId, ScheduledHandle>>>,
}

impl PushScheduler {
    pub fn new(
        store: Arc<DataPushStore>,
        event_bus: Option<Arc<EventBus>>,
        renderer: Arc<TemplateRenderer>,
    ) -> Self {
        Self {
            store,
            event_bus,
            renderer,
            handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a target's schedule.
    pub async fn start(&self, target: PushTarget) -> Result<()> {
        let target_id = target.id.clone();
        // Stop existing if running
        self.stop(&target_id).await;

        let (tx, rx) = tokio::sync::watch::channel(false);

        // Extract schedule info before moving target
        let schedule_info = match &target.schedule {
            PushSchedule::EventDriven { event_types } => {
                ScheduleInfo::EventDriven(event_types.clone())
            }
            PushSchedule::Interval { interval_secs } => ScheduleInfo::Interval(*interval_secs),
        };

        let handle = match schedule_info {
            ScheduleInfo::EventDriven(event_types) => {
                self.spawn_event_driven(target, event_types, rx)
            }
            ScheduleInfo::Interval(interval_secs) => self.spawn_interval(target, interval_secs, rx),
        };

        let mut handles = self.handles.write().await;
        handles.insert(
            target_id,
            ScheduledHandle {
                cancel: tx,
                join_handle: handle,
            },
        );

        Ok(())
    }

    /// Stop a running target.
    pub async fn stop(&self, target_id: &str) {
        // Remove under the lock, then drop the guard before awaiting the
        // task join. Otherwise a slow stop (e.g. retry backoff) would hold
        // the write lock and serialize all other target operations.
        let h = {
            let mut handles = self.handles.write().await;
            handles.remove(target_id)
        };
        if let Some(h) = h {
            h.stop().await;
        }
    }

    /// Stop all running targets.
    pub async fn stop_all(&self) {
        let mut handles = self.handles.write().await;
        let drained: Vec<_> = handles.drain().collect();
        drop(handles);
        for (_, h) in drained {
            h.stop().await;
        }
    }

    fn spawn_event_driven(
        &self,
        target: PushTarget,
        event_types: Vec<String>,
        mut cancel: tokio::sync::watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        let event_bus = self.event_bus.clone();
        let store = self.store.clone();
        let renderer = self.renderer.clone();

        tokio::spawn(async move {
            let Some(bus) = event_bus else {
                tracing::warn!(target_id = %target.id, "No event bus available for event-driven target");
                return;
            };

            let mut rx = bus.subscribe();
            let mut matcher = DataSourceMatcher::new(target.data_filter.clone());
            let dest = match create_destination(&target.target_type, &target.config) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(target_id = %target.id, error = %e, "Failed to create destination");
                    return;
                }
            };

            let batch_enabled = target.batch_config.is_enabled();
            let batch_size = target.batch_config.batch_size;
            let batch_interval =
                std::time::Duration::from_millis(target.batch_config.batch_interval_ms);

            tracing::info!(
                target_id = %target.id,
                batch_enabled,
                batch_size,
                batch_interval_ms = target.batch_config.batch_interval_ms,
                "Event-driven push target started"
            );

            // Buffer for batched events
            let mut buffer: Vec<(String, serde_json::Value, i64)> = Vec::new();
            let mut flush_timer = tokio::time::Instant::now() + batch_interval;

            loop {
                tokio::select! {
                    _ = cancel.changed() => {
                        // Flush remaining buffer before stopping
                        if !buffer.is_empty() {
                            flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, None).await;
                        }
                        tracing::info!(target_id = %target.id, "Event-driven target stopped");
                        return;
                    }
                    result = rx.recv() => {
                        if cancel.has_changed().unwrap_or(false) {
                            if !buffer.is_empty() {
                                flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, None).await;
                            }
                            return;
                        }
                        match result {
                            Some((event, _metadata)) => {
                                if !matches_event_type(&event, &event_types) {
                                    continue;
                                }
                                if let Some((source_id, value, ts)) = extract_event_data(&event) {
                                    let value_str = value.to_string();
                                    if matcher.should_push(&source_id, &value_str) {
                                        if !batch_enabled {
                                            // Immediate delivery (batch_size=1)
                                            if let Err(e) = deliver_with_retry(
                                                &target,
                                                &store,
                                                &renderer,
                                                dest.as_ref(),
                                                &source_id,
                                                &value,
                                                ts,
                                                Some(&cancel),
                                            ).await {
                                                tracing::warn!(target_id = %target.id, error = %e, "Delivery failed after retries");
                                            }
                                        } else {
                                            // Buffer for batch
                                            buffer.push((source_id, value, ts));
                                            if buffer.len() >= batch_size {
                                                flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, Some(&cancel)).await;
                                                flush_timer = tokio::time::Instant::now() + batch_interval;
                                            }
                                        }
                                    }
                                }
                            }
                            None => {
                                if !buffer.is_empty() {
                                    flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, None).await;
                                }
                                return;
                            }
                        }
                    }
                    _ = tokio::time::sleep_until(flush_timer), if batch_enabled && !buffer.is_empty() => {
                        flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, Some(&cancel)).await;
                        flush_timer = tokio::time::Instant::now() + batch_interval;
                    }
                }
            }
        })
    }

    fn spawn_interval(
        &self,
        target: PushTarget,
        interval_secs: u64,
        mut cancel: tokio::sync::watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        let event_bus = self.event_bus.clone();
        let store = self.store.clone();
        let renderer = self.renderer.clone();

        tokio::spawn(async move {
            let Some(bus) = event_bus else {
                tracing::warn!(target_id = %target.id, "No event bus available for interval target");
                return;
            };

            let dest = match create_destination(&target.target_type, &target.config) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(target_id = %target.id, error = %e, "Failed to create destination");
                    return;
                }
            };

            let mut rx = bus.subscribe();
            let mut matcher = DataSourceMatcher::new(target.data_filter.clone());
            let mut buffer: Vec<(String, serde_json::Value, i64)> = Vec::new();
            let flush_interval = std::time::Duration::from_secs(interval_secs);

            tracing::info!(target_id = %target.id, interval_secs, "Interval push target started");

            // Use interval as flush timer; collect events between ticks
            let mut flush_timer = tokio::time::interval(flush_interval);
            // Skip the first immediate tick
            flush_timer.tick().await;

            loop {
                tokio::select! {
                    _ = cancel.changed() => {
                        // Flush remaining buffer before stopping
                        if !buffer.is_empty() {
                            flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, None).await;
                        }
                        tracing::info!(target_id = %target.id, "Interval target stopped");
                        return;
                    }
                    result = rx.recv() => {
                        if cancel.has_changed().unwrap_or(false) {
                            if !buffer.is_empty() {
                                flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, None).await;
                            }
                            return;
                        }
                        if let Some((event, _metadata)) = result {
                            if let Some((source_id, value, ts)) = extract_event_data(&event) {
                                let value_str = value.to_string();
                                if matcher.should_push(&source_id, &value_str) {
                                    buffer.push((source_id, value, ts));
                                }
                            }
                        }
                    }
                    _ = flush_timer.tick() => {
                        if !buffer.is_empty() {
                            tracing::debug!(target_id = %target.id, count = buffer.len(), "Interval flush");
                            flush_batch(&target, &store, &renderer, dest.as_ref(), &mut buffer, Some(&cancel)).await;
                        }
                    }
                }
            }
        })
    }
}

/// Check if a HeraMindEvent matches any of the requested event types.
fn matches_event_type(event: &heramind_core::HeraMindEvent, event_types: &[String]) -> bool {
    if event_types.is_empty() {
        return true;
    }
    let type_name = match event {
        heramind_core::HeraMindEvent::DeviceMetric { .. } => "device_metric",
        heramind_core::HeraMindEvent::ExtensionOutput { .. } => "extension_output",
        heramind_core::HeraMindEvent::AlertCreated { .. } => "alert_created",
        _ => return false,
    };
    event_types.iter().any(|t| t == type_name)
}

/// Convert MetricValue to serde_json::Value.
fn metric_to_json(value: &heramind_core::MetricValue) -> serde_json::Value {
    match value {
        heramind_core::MetricValue::Float(f) => json!(*f),
        heramind_core::MetricValue::Integer(i) => json!(*i),
        heramind_core::MetricValue::Boolean(b) => json!(*b),
        heramind_core::MetricValue::String(s) => json!(s),
        heramind_core::MetricValue::Json(v) => v.clone(),
    }
}

/// Extract data from a HeraMindEvent for push delivery.
fn extract_event_data(
    event: &heramind_core::HeraMindEvent,
) -> Option<(String, serde_json::Value, i64)> {
    match event {
        heramind_core::HeraMindEvent::DeviceMetric {
            device_id,
            metric,
            value,
            timestamp,
            ..
        } => {
            let source_id = format!("device:{}:{}", device_id, metric);
            let val = metric_to_json(value);
            Some((source_id, val, *timestamp))
        }
        heramind_core::HeraMindEvent::ExtensionOutput {
            extension_id,
            output_name,
            value,
            timestamp,
            ..
        } => {
            let source_id = format!("extension:{}:{}", extension_id, output_name);
            let val = metric_to_json(value);
            Some((source_id, val, *timestamp))
        }
        _ => None,
    }
}

/// Deliver data with retry logic.
///
/// `cancel` — when `Some`, the inter-retry backoff sleeps are racing against
/// this watch receiver. As soon as the receiver observes a change, the
/// in-flight retry loop aborts and the function returns `Err`. This lets
/// `PushScheduler::stop` interrupt a target stuck in a long backoff tail
/// instead of blocking the stop call for the full backoff sum.
#[allow(clippy::too_many_arguments)]
async fn deliver_with_retry(
    target: &PushTarget,
    store: &DataPushStore,
    renderer: &TemplateRenderer,
    dest: &dyn crate::targets::PushDestination,
    source_id: &str,
    value: &serde_json::Value,
    timestamp: i64,
    cancel: Option<&tokio::sync::watch::Receiver<bool>>,
) -> Result<()> {
    let ctx = TemplateContext {
        source_id: source_id.to_string(),
        value: value.clone(),
        timestamp,
        metadata: None,
    };

    let payload = renderer.render(&target.template, &ctx)?;

    let log_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let mut log = DeliveryLog {
        id: log_id,
        target_id: target.id.clone(),
        status: DeliveryStatus::Pending,
        data_source_id: source_id.to_string(),
        payload_sent: payload.clone(),
        response: None,
        attempts: 0,
        created_at: now,
        completed_at: None,
        error: None,
    };

    let max_retries = target.retry_config.max_retries;
    let mut backoff = target.retry_config.backoff_secs;

    for attempt in 0..=max_retries {
        log.attempts = attempt + 1;
        match dest.send(&payload).await {
            Ok(()) => {
                log.status = DeliveryStatus::Success;
                log.completed_at = Some(chrono::Utc::now().timestamp());
                let _ = store.save_delivery_log(&log);
                tracing::debug!(target_id = %target.id, attempt, "Delivery successful");
                return Ok(());
            }
            Err(e) => {
                log.error = Some(e.to_string());
                if attempt < max_retries {
                    log.status = DeliveryStatus::Retrying;
                    let _ = store.save_delivery_log(&log);
                    let effective_backoff = backoff.min(target.retry_config.max_backoff_secs);
                    tracing::warn!(
                        target_id = %target.id,
                        attempt,
                        backoff_secs = effective_backoff,
                        error = %e,
                        "Delivery failed, retrying"
                    );
                    if sleep_or_cancel(
                        std::time::Duration::from_secs(effective_backoff),
                        cancel,
                    )
                    .await
                    {
                        log.status = DeliveryStatus::Failed;
                        log.error = Some(format!("Cancelled during retry backoff: {}", e));
                        log.completed_at = Some(chrono::Utc::now().timestamp());
                        let _ = store.save_delivery_log(&log);
                        tracing::info!(
                            target_id = %target.id,
                            attempt,
                            "Delivery retry aborted by stop signal"
                        );
                        return Err(anyhow::anyhow!("Cancelled during retry backoff: {}", e));
                    }
                    backoff = backoff.saturating_mul(2);
                } else {
                    log.status = DeliveryStatus::Failed;
                    log.completed_at = Some(chrono::Utc::now().timestamp());
                    let _ = store.save_delivery_log(&log);
                    return Err(e);
                }
            }
        }
    }

    Err(anyhow::anyhow!("Max retries exceeded"))
}

/// Sleep for `dur`, returning early when `cancel` observes a value change.
/// Returns `true` if cancelled (returned via the watch), `false` on natural
/// completion. When `cancel` is `None`, this is a plain sleep.
async fn sleep_or_cancel(
    dur: std::time::Duration,
    cancel: Option<&tokio::sync::watch::Receiver<bool>>,
) -> bool {
    let Some(rx) = cancel else {
        tokio::time::sleep(dur).await;
        return false;
    };
    // Use a borrowed clone of the watch receiver so the parent's
    // last_observed version is untouched (cloning a Receiver creates an
    // independent version cursor).
    let mut rx = rx.clone();
    tokio::select! {
        _ = tokio::time::sleep(dur) => false,
        _ = rx.changed() => true,
    }
}

/// Flush a batch of buffered events as a single aggregated payload.
///
/// `cancel` semantics mirror [`deliver_with_retry`]: when `Some`, an
/// in-flight backoff is aborted on stop. The final-flush call paths inside
/// the `select!` cancel arms pass `None` because the task is already
/// tearing down and we want the flush to complete unconditionally.
async fn flush_batch(
    target: &PushTarget,
    store: &DataPushStore,
    renderer: &TemplateRenderer,
    dest: &dyn crate::targets::PushDestination,
    buffer: &mut Vec<(String, serde_json::Value, i64)>,
    cancel: Option<&tokio::sync::watch::Receiver<bool>>,
) {
    if buffer.is_empty() {
        return;
    }

    let items: Vec<serde_json::Value> = buffer
        .iter()
        .map(|(source_id, value, ts)| {
            let ctx = TemplateContext {
                source_id: source_id.clone(),
                value: value.clone(),
                timestamp: *ts,
                metadata: None,
            };
            // Try to render each item; fall back to raw JSON
            renderer
                .render(&target.template, &ctx)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| json!({"source_id": source_id, "value": value, "timestamp": ts, "metadata": null}))
        })
        .collect();

    let count = items.len();
    let source_ids: Vec<&str> = buffer.iter().map(|(s, _, _)| s.as_str()).collect();

    let batch_payload = json!({
        "batch": true,
        "count": count,
        "items": items,
    });

    let payload_str = serde_json::to_string(&batch_payload).unwrap_or_default();

    let log_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let mut log = DeliveryLog {
        id: log_id,
        target_id: target.id.clone(),
        status: DeliveryStatus::Pending,
        data_source_id: source_ids.join(","),
        payload_sent: payload_str.clone(),
        response: None,
        attempts: 0,
        created_at: now,
        completed_at: None,
        error: None,
    };

    let max_retries = target.retry_config.max_retries;
    let mut backoff = target.retry_config.backoff_secs;

    for attempt in 0..=max_retries {
        log.attempts = attempt + 1;
        match dest.send(&payload_str).await {
            Ok(()) => {
                log.status = DeliveryStatus::Success;
                log.completed_at = Some(chrono::Utc::now().timestamp());
                let _ = store.save_delivery_log(&log);
                tracing::debug!(
                    target_id = %target.id,
                    batch_count = count,
                    attempt,
                    "Batch delivery successful"
                );
                buffer.clear();
                return;
            }
            Err(e) => {
                log.error = Some(e.to_string());
                if attempt < max_retries {
                    log.status = DeliveryStatus::Retrying;
                    let _ = store.save_delivery_log(&log);
                    let effective_backoff = backoff.min(target.retry_config.max_backoff_secs);
                    tracing::warn!(
                        target_id = %target.id,
                        batch_count = count,
                        attempt,
                        backoff_secs = effective_backoff,
                        error = %e,
                        "Batch delivery failed, retrying"
                    );
                    if sleep_or_cancel(
                        std::time::Duration::from_secs(effective_backoff),
                        cancel,
                    )
                    .await
                    {
                        log.status = DeliveryStatus::Failed;
                        log.error = Some(format!("Cancelled during retry backoff: {}", e));
                        log.completed_at = Some(chrono::Utc::now().timestamp());
                        let _ = store.save_delivery_log(&log);
                        tracing::info!(
                            target_id = %target.id,
                            batch_count = count,
                            attempt,
                            "Batch delivery retry aborted by stop signal"
                        );
                        buffer.clear();
                        return;
                    }
                    backoff = backoff.saturating_mul(2);
                } else {
                    log.status = DeliveryStatus::Failed;
                    log.completed_at = Some(chrono::Utc::now().timestamp());
                    let _ = store.save_delivery_log(&log);
                    tracing::warn!(
                        target_id = %target.id,
                        batch_count = count,
                        error = %e,
                        "Batch delivery failed after retries"
                    );
                    buffer.clear();
                    return;
                }
            }
        }
    }

    buffer.clear();
}
