use super::*;
use std::{fs, os::unix::fs::PermissionsExt};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct TempRoot(PathBuf);
impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "heramind-engine-test-{}-{}",
            std::process::id(),
            TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("assets")).unwrap();
        Self(path)
    }
    fn script(&self, body: &str) -> PathBuf {
        let path = self.0.join("assets/engine-worker");
        let staging = self.0.join("worker-staging");
        fs::write(&staging, format!("#!/usr/bin/python3\n{body}")).unwrap();
        fs::set_permissions(&staging, fs::Permissions::from_mode(0o700)).unwrap();
        fs::rename(staging, &path).unwrap();
        path
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn request() -> Value {
    json!({"protocol": 1, "request_id": "camera-A-1", "task": "health", "args": {}})
}
const READ_REQUEST: &str = "import sys, struct, json, time\nn = struct.unpack('!I', sys.stdin.buffer.read(4))[0]\nr = json.loads(sys.stdin.buffer.read(n))\n";

#[tokio::test]
async fn rejects_wrong_request_id_and_oversized_header() {
    for body in [
        "out = json.dumps({'protocol': 1, 'request_id': 'another-camera', 'ok': True}).encode()\nsys.stdout.buffer.write(struct.pack('!I', len(out)) + out)\nsys.stdout.buffer.flush()\n",
        "sys.stdout.buffer.write(struct.pack('!I', 2097153))\nsys.stdout.buffer.flush()\ntime.sleep(2)\n",
    ] {
        let root = TempRoot::new();
        let path = root.script(&format!("{READ_REQUEST}{body}"));
        let mut worker = Worker::test_peer(&path, &root.0).unwrap();
        assert!(worker.exchange(&request(), 1000).await.is_err());
        assert!(!worker.synchronized);
        worker.shutdown().await;
    }
}

#[tokio::test]
async fn timeout_and_cancellation_poison_stream() {
    let root = TempRoot::new();
    let path = root.script(&format!("{READ_REQUEST}time.sleep(10)\n"));
    let mut worker = Worker::test_peer(&path, &root.0).unwrap();
    assert!(worker
        .exchange(&request(), 50)
        .await
        .unwrap_err()
        .to_string()
        .contains("timed out"));
    assert!(!worker.synchronized);
    worker.shutdown().await;

    let mut worker = Worker::test_peer(&path, &root.0).unwrap();
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(50),
        worker.exchange(&request(), 5000)
    )
    .await
    .is_err());
    assert!(!worker.synchronized);
    worker.shutdown().await;
}

#[tokio::test]
async fn eof_is_a_transport_failure_but_task_error_keeps_protocol_usable() {
    let root = TempRoot::new();
    let path = root.script(READ_REQUEST);
    let mut worker = Worker::test_peer(&path, &root.0).unwrap();
    assert!(worker.exchange(&request(), 1000).await.is_err());
    assert!(!worker.synchronized);
    worker.shutdown().await;
    let path = root.script(&format!("{READ_REQUEST}out = json.dumps({{'protocol': 1, 'request_id': r['request_id'], 'ok': False, 'error': 'missing model'}}).encode()\nsys.stdout.buffer.write(struct.pack('!I', len(out)) + out)\nsys.stdout.buffer.flush()\n"));
    let mut worker = Worker::test_peer(&path, &root.0).unwrap();
    let response = worker.exchange(&request(), 1000).await.unwrap();
    assert_eq!(response["ok"], false);
    assert!(worker.synchronized);
    worker.shutdown().await;
}

#[tokio::test]
async fn bindings_validate_and_filter_metric_events() {
    let mut bridge = EngineBridge::new();
    let binding = json!({"device_id": "camera-A", "metric": "image", "task": "image.inspect"});
    bridge
        .configure(&json!({"bindings": [binding.clone()]}))
        .await
        .unwrap();
    let event = json!({"device_id": "camera-A", "metric": "image", "timestamp": 123,
        "value": {"String": "data:image/png;base64,AAAA"}, "is_virtual": false});
    let args = bridge.event_args("DeviceMetric", &event).unwrap();
    assert_eq!(args["device_id"], "camera-A");
    assert_eq!(args["source_timestamp"], 123);
    assert!(bridge.event_args("ExtensionOutput", &event).is_none());
    let mut wrong_device = event.clone();
    wrong_device["device_id"] = json!("camera-B");
    assert!(bridge.event_args("DeviceMetric", &wrong_device).is_none());
    let mut virtual_event = event.clone();
    virtual_event["is_virtual"] = json!(true);
    assert!(bridge.event_args("DeviceMetric", &virtual_event).is_none());
    assert!(bridge
        .event_args(
            "DeviceMetric",
            &json!({"event_type": "DeviceMetric", "payload": event})
        )
        .is_some());
    assert!(bridge
        .configure(&json!({"bindings": [binding.clone(), binding]}))
        .await
        .is_err());
    assert!(bridge
        .configure(&json!({"bindings": [{"device_id": "a", "metric": "virtual.engine.result"}]}))
        .await
        .is_err());
    assert!(bridge.configure(&json!({"timeout_ms": 0})).await.is_err());
    // Invalid configuration must not erase the last working configuration.
    assert_eq!(bridge.state.lock().unwrap().config.bindings.len(), 1);
}

#[tokio::test]
async fn concurrent_inputs_fail_fast_instead_of_queuing() {
    let bridge = EngineBridge::new();
    let _guard = bridge.worker.lock().await;
    let error = bridge.call("health", &json!({})).await.unwrap_err();
    assert!(error.to_string().contains("busy"));
    assert_eq!(bridge.dropped.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn camera_sampling_drops_repeated_frames_and_resets_on_configure() {
    let mut bridge = EngineBridge::new();
    let config =
        json!({"bindings": [{"device_id": "a", "metric": "image", "min_interval_ms": 10000}]});
    bridge.configure(&config).await.unwrap();
    let event = json!({"device_id": "a", "metric": "image", "value": {"String": "AAAA"}});
    assert!(bridge.event_args("DeviceMetric", &event).is_some());
    assert!(bridge.event_args("DeviceMetric", &event).is_none());
    assert_eq!(bridge.dropped.load(Ordering::Relaxed), 1);
    bridge.configure(&config).await.unwrap();
    assert!(bridge.event_args("DeviceMetric", &event).is_some());
}

// Set HERAMIND_TEST_ENGINE_WORKER to the compiled C++ worker and explicitly
// run ignored tests to exercise the full Rust → engine node → Rust path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn real_engine_roundtrip() {
    let binary = PathBuf::from(
        std::env::var_os("HERAMIND_TEST_ENGINE_WORKER").expect("Set HERAMIND_TEST_ENGINE_WORKER"),
    );
    let root = TempRoot::new();
    let installed = root.0.join("assets/engine-worker");
    fs::copy(binary.canonicalize().unwrap(), &installed).unwrap();
    // Simulate the platform's package extractor dropping executable bits.
    fs::set_permissions(&installed, fs::Permissions::from_mode(0o600)).unwrap();
    std::env::set_var("HERAMIND_EXTENSION_DIR", &root.0);
    let bridge = EngineBridge::new();
    let health = bridge.execute_command("health", &json!({})).await.unwrap();
    assert_eq!(health["data"]["backend"], "cpu");
    assert_eq!(health["data"]["qnn_available"], false);
    // One black pixel PNG, independently decoded by OpenCV in the actual worker.
    let image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGNgYGAAAAAEAAH2FzhVAAAAAElFTkSuQmCC";
    let args = json!({"device_id": "camera-A", "image_base64": image, "source_timestamp": 123});
    let a = bridge.execute_command("process", &args).await.unwrap();
    assert_eq!(a["device_id"], "camera-A");
    assert_eq!(a["source_timestamp"], 123);
    assert_eq!(a["result"]["width"], 1);
    let mut b = args.clone();
    b["device_id"] = json!("camera-B");
    b["task"] = json!("image.preprocess");
    b["options"] = json!({"width": 24, "height": 16, "format": "gray"});
    let b = bridge.execute_command("process", &b).await.unwrap();
    assert_ne!(a["request_id"], b["request_id"]);
    assert_eq!(b["result"]["output_width"], 24);
    assert!(b["result"]["image_base64"]
        .as_str()
        .unwrap()
        .starts_with("data:image/jpeg;base64,"));
    let latest = bridge
        .execute_command("get_latest", &json!({"device_id": "camera-B"}))
        .await
        .unwrap();
    assert!(latest["latest"]["result"].get("image_base64").is_none());
    let bad = json!({"device_id": "camera-A", "image_base64": "invalid"});
    assert!(bridge.execute_command("process", &bad).await.is_err());
    let mut qnn = args.clone();
    qnn["options"] = json!({"backend": "qnn"});
    assert!(bridge
        .execute_command("process", &qnn)
        .await
        .unwrap_err()
        .to_string()
        .contains("unavailable"));
    assert!(bridge.execute_command("process", &args).await.is_ok());
    bridge.on_unload().await.unwrap();
    std::env::remove_var("HERAMIND_EXTENSION_DIR");
}
