mod transport;

use async_trait::async_trait;
use heramind_extension_sdk::{
    capabilities::CapabilityContext,
    events::{MetricValueData, SdkEvent},
    heramind_export, CommandBuilder, Extension, ExtensionCommand, ExtensionError,
    ExtensionMetadata, ExtensionMetricValue, MetricBuilder, MetricDataType, MetricDescriptor,
    ParamMetricValue, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex as AsyncMutex;
use transport::Worker;

const MAX_REQUEST: usize = 8 * 1024 * 1024;
const MAX_RESPONSE: usize = 2 * 1024 * 1024;
const MAX_BINDINGS: usize = 64;

fn failure(message: impl ToString) -> ExtensionError {
    ExtensionError::ExecutionFailed(message.to_string())
}
fn invalid(message: impl ToString) -> ExtensionError {
    ExtensionError::InvalidArguments(message.to_string())
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    device_id: String,
    #[serde(default = "image_metric")]
    metric: String,
    #[serde(default = "inspect_task")]
    task: String,
    #[serde(default)]
    options: Value,
    #[serde(default = "enabled")]
    enabled: bool,
    #[serde(default)]
    min_interval_ms: u64,
}
fn image_metric() -> String {
    "image".into()
}
fn inspect_task() -> String {
    "image.inspect".into()
}
fn enabled() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Config {
    timeout_ms: u64,
    bindings: Vec<Binding>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            timeout_ms: 10_000,
            bindings: vec![],
        }
    }
}

struct State {
    config: Config,
    // At most one latest result per binding/device; never retain source images.
    latest: BTreeMap<String, Value>,
    last_started: BTreeMap<(String, String), Instant>,
    last_error: Option<String>,
}

pub struct EngineBridge {
    worker: AsyncMutex<Option<Worker>>,
    state: Mutex<State>,
    sequence: AtomicU64,
    instance_id: u64,
    processed: AtomicU64,
    failed: AtomicU64,
    dropped: AtomicU64,
    publish_failed: AtomicU64,
}

impl Default for EngineBridge {
    fn default() -> Self {
        Self::new()
    }
}
impl EngineBridge {
    pub fn new() -> Self {
        Self {
            worker: AsyncMutex::new(None),
            state: Mutex::new(State {
                config: Config::default(),
                latest: BTreeMap::new(),
                last_started: BTreeMap::new(),
                last_error: None,
            }),
            sequence: AtomicU64::new(0),
            instance_id: now_ms(),
            processed: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            publish_failed: AtomicU64::new(0),
        }
    }

    fn root() -> Result<PathBuf> {
        let path = std::env::var_os("HERAMIND_EXTENSION_DIR")
            .or_else(|| std::env::var_os("NEOMIND_EXTENSION_DIR"))
            .ok_or_else(|| failure("HERAMIND_EXTENSION_DIR is missing"))?;
        PathBuf::from(path).canonicalize().map_err(failure)
    }

    fn validate_binding(binding: &Binding) -> Result<()> {
        if binding.min_interval_ms > 60_000 {
            return Err(invalid("min_interval_ms must be 0–60000"));
        }
        if binding.device_id.is_empty()
            || binding.device_id.len() > 256
            || binding.metric.is_empty()
            || binding.metric.len() > 256
            || binding.metric.starts_with("virtual.engine.")
        {
            return Err(invalid(
                "Invalid device/metric or recursive engine output binding",
            ));
        }
        if !matches!(
            binding.task.as_str(),
            "image.inspect" | "image.preprocess" | "face.recognize"
        ) {
            return Err(invalid(
                "Binding task must be image.inspect, image.preprocess or face.recognize",
            ));
        }
        if !binding.options.is_null() && !binding.options.is_object() {
            return Err(invalid("Binding options must be an object"));
        }
        if serde_json::to_vec(&binding.options).map_err(invalid)?.len() > 4096 {
            return Err(invalid("Binding options exceed 4 KiB"));
        }
        Ok(())
    }

    async fn call(&self, task: &str, args: &Value) -> Result<Value> {
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        let request_id = format!("{}-{}-{seq}", std::process::id(), self.instance_id);
        let request = json!({"protocol": 1, "request_id": request_id, "task": task, "args": args});
        // Serialize/limit before spawning or touching the protocol stream.
        if serde_json::to_vec(&request).map_err(invalid)?.len() > MAX_REQUEST {
            return Err(invalid("Engine request exceeds 8 MiB"));
        }
        let timeout_ms = self.state.lock().unwrap().config.timeout_ms;
        let mut guard = self.worker.try_lock().map_err(|_| {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            failure("Engine busy; input dropped (no unbounded queue)")
        })?;
        if guard.as_ref().is_some_and(|w| !w.synchronized) {
            if let Some(mut worker) = guard.take() {
                worker.shutdown().await;
            }
        }
        if guard.is_none() {
            match Self::root()
                .and_then(|root| Worker::start(&root.join("assets/engine-worker"), &root))
            {
                Ok(worker) => *guard = Some(worker),
                Err(error) => {
                    self.failed.fetch_add(1, Ordering::Relaxed);
                    self.state.lock().unwrap().last_error = Some(error.to_string());
                    return Err(error);
                }
            }
        }
        let response = guard.as_mut().unwrap().exchange(&request, timeout_ms).await;
        match response {
            Ok(response) if response["ok"] == true => {
                self.processed.fetch_add(1, Ordering::Relaxed);
                self.state.lock().unwrap().last_error = None;
                Ok(json!({"request_id": request_id, "data": response["data"]}))
            }
            other => {
                let error = match other {
                    Ok(response) => failure(
                        response["error"]
                            .as_str()
                            .unwrap_or("Engine rejected input"),
                    ),
                    Err(error) => {
                        if let Some(mut worker) = guard.take() {
                            worker.shutdown().await;
                        }
                        error
                    }
                };
                self.failed.fetch_add(1, Ordering::Relaxed);
                self.state.lock().unwrap().last_error = Some(error.to_string());
                Err(error)
            }
        }
    }

    async fn process(&self, args: &Value) -> Result<Value> {
        let device = args["device_id"]
            .as_str()
            .filter(|v| !v.is_empty() && v.len() <= 256)
            .ok_or_else(|| invalid("device_id is required (1–256 bytes)"))?;
        let task = args["task"].as_str().unwrap_or("image.inspect");
        if !matches!(
            task,
            "image.inspect" | "image.preprocess" | "face.recognize"
        ) {
            return Err(invalid("Unsupported processing task"));
        }
        let received = now_ms();
        let start = Instant::now();
        let response = self.call(task, args).await?;
        let result = json!({
            "request_id": response["request_id"], "device_id": device,
            "source_timestamp": args.get("source_timestamp"), "task": task,
            "received_at_ms": received, "completed_at_ms": now_ms(),
            "latency_ms": start.elapsed().as_secs_f64() * 1000.0, "result": response["data"]
        });
        let mut state = self.state.lock().unwrap();
        if !state.latest.contains_key(device) && state.latest.len() >= MAX_BINDINGS {
            // Cache is for inspection, not an ever-growing history store.
            if let Some(key) = state.latest.keys().next().cloned() {
                state.latest.remove(&key);
            }
        }
        // Processed image bytes can be large. Return them only to the caller.
        let mut cached = result.clone();
        if let Some(obj) = cached["result"].as_object_mut() {
            obj.remove("image_base64");
        }
        state.latest.insert(device.into(), cached);
        Ok(result)
    }

    fn event_args(&self, event_type: &str, payload: &Value) -> Option<Value> {
        let SdkEvent::DeviceMetric(event) = SdkEvent::parse(event_type, payload) else {
            return None;
        };
        if event.is_virtual {
            return None;
        }
        let MetricValueData::String(image) = event.value else {
            return None;
        };
        let mut state = self.state.lock().unwrap();
        let binding = state
            .config
            .bindings
            .iter()
            .find(|b| b.enabled && b.device_id == event.device_id && b.metric == event.metric)
            .cloned()?;
        let key = (event.device_id.clone(), event.metric.clone());
        if state
            .last_started
            .get(&key)
            .is_some_and(|start| start.elapsed().as_millis() < u128::from(binding.min_interval_ms))
        {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        state.last_started.insert(key, Instant::now());
        let timestamp = event
            .timestamp
            .or_else(|| payload.get("timestamp").and_then(Value::as_i64));
        Some(
            json!({"device_id": event.device_id, "source_timestamp": timestamp,
            "image_base64": image, "task": binding.task, "options": binding.options}),
        )
    }

    async fn shutdown(&self) {
        let mut guard = self.worker.lock().await;
        if let Some(mut worker) = guard.take() {
            worker.shutdown().await;
        }
    }
}

#[async_trait]
impl Extension for EngineBridge {
    fn metadata(&self) -> &ExtensionMetadata {
        static META: OnceLock<ExtensionMetadata> = OnceLock::new();
        META.get_or_init(|| {
            ExtensionMetadata::new("engine-bridge", "HeraMind Engine Bridge", "0.1.0")
                .with_description(
                    "Dedicated CVEDIX engine worker for HeraMind image inputs and SeetaFace",
                )
        })
    }

    fn commands(&self) -> Vec<ExtensionCommand> {
        vec![
            CommandBuilder::new("health")
                .display_name("Engine capabilities")
                .build(),
            CommandBuilder::new("get_status")
                .display_name("Engine bridge status")
                .build(),
            CommandBuilder::new("get_latest")
                .param_simple("device_id", "Device ID", MetricDataType::String)
                .build(),
            CommandBuilder::new("process")
                .display_name("Process an input image")
                .param_simple("device_id", "Device ID", MetricDataType::String)
                .param_simple(
                    "image_base64",
                    "JPEG/PNG/WebP base64 or data URI",
                    MetricDataType::String,
                )
                .param_simple(
                    "task",
                    "image.inspect / image.preprocess / face.recognize",
                    MetricDataType::String,
                )
                .build(),
            CommandBuilder::new("face_register")
                .param_simple("name", "Person name", MetricDataType::String)
                .param_simple(
                    "image_base64",
                    "Image with exactly one face",
                    MetricDataType::String,
                )
                .build(),
            CommandBuilder::new("face_delete")
                .param_simple("face_id", "Face ID", MetricDataType::Integer)
                .build(),
        ]
    }

    async fn execute_command(&self, command: &str, args: &Value) -> Result<Value> {
        match command {
            "health" => self.call("health", &json!({})).await,
            "process" => self.process(args).await,
            "face_register" => self.call("face.register", args).await,
            "face_delete" => self.call("face.delete", args).await,
            "get_latest" => {
                let device = args["device_id"]
                    .as_str()
                    .ok_or_else(|| invalid("device_id is required"))?;
                Ok(json!({"latest": self.state.lock().unwrap().latest.get(device)}))
            }
            "get_status" => {
                let state = self.state.lock().unwrap();
                Ok(
                    json!({"config": state.config, "last_error": state.last_error,
                    "processed": self.processed.load(Ordering::Relaxed),
                    "failed": self.failed.load(Ordering::Relaxed),
                    "dropped": self.dropped.load(Ordering::Relaxed),
                    "publish_failed": self.publish_failed.load(Ordering::Relaxed)}),
                )
            }
            _ => Err(ExtensionError::CommandNotFound(command.into())),
        }
    }

    async fn configure(&mut self, value: &Value) -> Result<()> {
        let config: Config = serde_json::from_value(value.clone()).map_err(invalid)?;
        if !(100..=60_000).contains(&config.timeout_ms) || config.bindings.len() > MAX_BINDINGS {
            return Err(invalid(
                "timeout_ms must be 100–60000 and at most 64 bindings are allowed",
            ));
        }
        let mut keys = std::collections::BTreeSet::new();
        for binding in &config.bindings {
            Self::validate_binding(binding)?;
            if !keys.insert((&binding.device_id, &binding.metric)) {
                return Err(invalid("Duplicate device/metric binding"));
            }
        }
        self.shutdown().await;
        let mut state = self.state.lock().unwrap();
        state.config = config;
        state.latest.clear();
        state.last_started.clear();
        Ok(())
    }

    fn event_subscriptions(&self) -> &[&str] {
        &["DeviceMetric"]
    }

    fn handle_event(&self, event_type: &str, payload: &Value) -> Result<()> {
        let Some(args) = self.event_args(event_type, payload) else {
            return Ok(());
        };
        // The SDK's native runner uses a multithread Tokio runtime. Do not block
        // its reactor while the engine subprocess runs or host capabilities reply.
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| failure("Engine events require the native runner Tokio runtime"))?;
        if handle.runtime_flavor() != tokio::runtime::RuntimeFlavor::MultiThread {
            return Err(failure("Engine events require a multithread runtime"));
        }
        tokio::task::block_in_place(|| {
            let result = handle.block_on(self.process(&args))?;
            let mut compact = result;
            if let Some(obj) = compact["result"].as_object_mut() {
                obj.remove("image_base64");
            }
            let response = CapabilityContext::default().invoke_capability(
                "event_publish",
                &json!({"event_type": "engine.result", "payload": compact}),
            );
            if response.get("success").and_then(Value::as_bool) != Some(true) {
                self.publish_failed.fetch_add(1, Ordering::Relaxed);
                self.state.lock().unwrap().last_error =
                    Some("Host event publication failed".into());
                return Err(failure(
                    "Engine result cached, but host event publication failed",
                ));
            }
            Ok(())
        })
    }

    fn metrics(&self) -> Vec<MetricDescriptor> {
        ["processed", "failed", "dropped", "publish_failed"]
            .iter()
            .map(|n| MetricBuilder::new(*n, *n).integer().unit("count").build())
            .collect()
    }
    fn produce_metrics(&self) -> Result<Vec<ExtensionMetricValue>> {
        Ok([
            ("processed", &self.processed),
            ("failed", &self.failed),
            ("dropped", &self.dropped),
            ("publish_failed", &self.publish_failed),
        ]
        .iter()
        .map(|(name, counter)| {
            ExtensionMetricValue::new(
                *name,
                ParamMetricValue::Integer(counter.load(Ordering::Relaxed) as i64),
            )
        })
        .collect())
    }
    async fn health_check(&self) -> Result<bool> {
        self.call("health", &json!({})).await.map(|_| true)
    }
    async fn on_unload(&self) -> Result<()> {
        self.shutdown().await;
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        // Dropping tokio Child uses kill_on_drop. on_unload also waits for exit.
        self.worker.get_mut().take();
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

heramind_export!(EngineBridge);

#[cfg(test)]
mod tests;
