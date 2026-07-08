//! Integration Tests for Storage System
//!
//! Tests cover:
//! - Time series storage
//! - Session storage
//! - Vector storage
//! - Performance tests

use chrono::Utc;
use heramind_storage::{
    DataPoint, SessionMessage, SessionStore, TimeSeriesStore, VectorDocument, VectorStore,
};
use tempfile::TempDir;

// ============================================================================
// Time Series Storage Tests
// ============================================================================

#[tokio::test]
async fn test_timeseries_basic_write_read() {
    let store = TimeSeriesStore::memory().unwrap();

    let point = DataPoint::new(Utc::now().timestamp(), 25.5);

    store
        .write("device1", "temperature", point.clone())
        .await
        .unwrap();

    store.flush().unwrap();

    let results = store
        .query_range(
            "device1",
            "temperature",
            point.timestamp - 1,
            point.timestamp + 1,
            None,
        )
        .await
        .unwrap();

    assert_eq!(results.points.len(), 1);
}

#[tokio::test]
async fn test_timeseries_batch_write() {
    let store = TimeSeriesStore::memory().unwrap();

    let now = Utc::now().timestamp();
    let points: Vec<DataPoint> = (0..10).map(|i| DataPoint::new(now + i, i as f64)).collect();

    store
        .write_batch("device1", "counter", points.clone())
        .await
        .unwrap();

    let results = store
        .query_range("device1", "counter", now - 1, now + 10, None)
        .await
        .unwrap();

    assert_eq!(results.points.len(), 10);
}

#[tokio::test]
async fn test_timeseries_query_range() {
    let store = TimeSeriesStore::memory().unwrap();

    let now = Utc::now().timestamp();

    // Write points at different times
    for i in 0..20 {
        let point = DataPoint::new(now + i * 60, i as f64);
        store.write("device1", "metric", point).await.unwrap();
    }

    store.flush().unwrap();

    // Query a specific range (inclusive)
    let results = store
        .query_range("device1", "metric", now + 5 * 60, now + 15 * 60, None)
        .await
        .unwrap();

    // Should get points 5-15 (11 points, inclusive range)
    assert!(results.points.len() >= 10);
}

#[tokio::test]
async fn test_timeseries_multiple_devices() {
    let store = TimeSeriesStore::memory().unwrap();

    let now = Utc::now().timestamp();

    // Write to multiple devices
    for device in ["device1", "device2", "device3"] {
        let point = DataPoint::new(now, 100.0);
        store.write(device, "status", point).await.unwrap();
    }

    store.flush().unwrap();

    // Query each device
    for device in ["device1", "device2", "device3"] {
        let results = store
            .query_range(device, "status", now - 1, now + 1, None)
            .await
            .unwrap();
        assert_eq!(results.points.len(), 1);
    }
}

#[tokio::test]
async fn test_timeseries_empty_query() {
    let store = TimeSeriesStore::memory().unwrap();

    let results = store
        .query_range("nonexistent", "metric", 0, Utc::now().timestamp(), None)
        .await
        .unwrap();

    assert_eq!(results.points.len(), 0);
}

// ============================================================================
// Session Storage Tests
// ============================================================================

#[test]
fn test_session_basic() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("session1.redb");
    let store = SessionStore::open(&db_path).unwrap();

    let session_id = "session-1";
    store.save_session_id(session_id).unwrap();

    let messages = vec![
        SessionMessage::user("Hello"),
        SessionMessage::assistant("Hi there!"),
    ];
    store.save_history(session_id, &messages).unwrap();

    let retrieved = store.load_history(session_id).unwrap();
    assert_eq!(retrieved.len(), 2);
}

#[test]
fn test_session_multiple_messages() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("session2.redb");
    let store = SessionStore::open(&db_path).unwrap();

    let session_id = "session-2";

    let messages: Vec<SessionMessage> = (0..10)
        .map(|i| {
            if i % 2 == 0 {
                SessionMessage::user(format!("User message {}", i))
            } else {
                SessionMessage::assistant(format!("Assistant message {}", i))
            }
        })
        .collect();

    store.save_session_id(session_id).unwrap();
    store.save_history(session_id, &messages).unwrap();

    let retrieved = store.load_history(session_id).unwrap();
    assert_eq!(retrieved.len(), 10);
}

#[test]
fn test_session_delete() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("session3.redb");
    let store = SessionStore::open(&db_path).unwrap();

    let session_id = "session-3";

    let messages = vec![
        SessionMessage::user("Message 1"),
        SessionMessage::assistant("Response 1"),
    ];
    store.save_session_id(session_id).unwrap();
    store.save_history(session_id, &messages).unwrap();

    store.delete_session(session_id).unwrap();

    let retrieved = store.load_history(session_id).unwrap();
    assert_eq!(retrieved.len(), 0);
}

#[test]
fn test_session_multiple_sessions() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("session4.redb");
    let store = SessionStore::open(&db_path).unwrap();

    for session_num in 0..3 {
        let session_id = format!("session-{}", session_num);
        let messages: Vec<SessionMessage> = (0..5)
            .map(|i| SessionMessage::user(format!("Session {} message {}", session_num, i)))
            .collect();
        store.save_session_id(&session_id).unwrap();
        store.save_history(&session_id, &messages).unwrap();
    }

    for session_num in 0..3 {
        let session_id = format!("session-{}", session_num);
        let retrieved = store.load_history(&session_id).unwrap();
        assert_eq!(retrieved.len(), 5);
    }
}

#[test]
fn test_session_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("session5.redb");
    let store = SessionStore::open(&db_path).unwrap();

    // Create the history table by saving a message first
    let messages = vec![SessionMessage::user("init")];
    store.save_session_id("init_session").unwrap();
    store.save_history("init_session", &messages).unwrap();

    // Now load a nonexistent session - should return empty
    let retrieved = store.load_history("nonexistent").unwrap();
    assert_eq!(retrieved.len(), 0);
}

// ============================================================================
// Vector Storage Tests
// ============================================================================

#[tokio::test]
async fn test_vector_basic_insert_search() {
    let store = VectorStore::new();

    let doc = VectorDocument::new("doc1", vec![0.1, 0.2, 0.3, 0.4]);
    store.insert(doc).await.unwrap();

    let query = vec![0.1, 0.2, 0.3, 0.4];
    let results = store.search(&query, 10).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "doc1");
}

#[tokio::test]
async fn test_vector_multiple_documents() {
    let store = VectorStore::new();

    for i in 0..10 {
        let embedding: Vec<f32> = (0..4).map(|j| (i * 4 + j) as f32 / 100.0).collect();
        let doc = VectorDocument::new(format!("doc{}", i), embedding);
        store.insert(doc).await.unwrap();
    }

    let query = vec![0.0, 0.01, 0.02, 0.03];
    let results = store.search(&query, 5).await.unwrap();

    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_vector_delete() {
    let store = VectorStore::new();

    let doc = VectorDocument::new("doc1", vec![0.1, 0.2, 0.3, 0.4]);
    store.insert(doc).await.unwrap();

    let _ = store.delete("doc1");

    let query = vec![0.1, 0.2, 0.3, 0.4];
    let results = store.search(&query, 10).await.unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_vector_update() {
    let store = VectorStore::new();

    let doc1 = VectorDocument::new("doc1", vec![0.1, 0.2, 0.3, 0.4]);
    store.insert(doc1).await.unwrap();

    let doc2 = VectorDocument::new("doc1", vec![0.5, 0.6, 0.7, 0.8]);
    store.insert(doc2).await.unwrap();

    let query = vec![0.5, 0.6, 0.7, 0.8];
    let results = store.search(&query, 10).await.unwrap();

    assert_eq!(results.len(), 1);
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
async fn test_timeseries_performance_write() {
    let store = TimeSeriesStore::memory().unwrap();

    let now = Utc::now().timestamp();
    let count = 100;

    for i in 0..count {
        let point = DataPoint::new(now + i, i as f64);
        store.write("perf-device", "metric", point).await.unwrap();
    }

    store.flush().unwrap();

    let results = store
        .query_range("perf-device", "metric", now, now + count, None)
        .await
        .unwrap();

    assert_eq!(results.points.len(), count as usize);
}

#[test]
fn test_session_performance() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("perf_session.redb");
    let store = SessionStore::open(&db_path).unwrap();

    let count = 100;

    for session_num in 0..count {
        let session_id = format!("perf-session-{}", session_num);
        let messages = vec![
            SessionMessage::user("Test message"),
            SessionMessage::assistant("Test response"),
        ];
        store.save_session_id(&session_id).unwrap();
        store.save_history(&session_id, &messages).unwrap();
    }

    for session_num in 0..count {
        let session_id = format!("perf-session-{}", session_num);
        let retrieved = store.load_history(&session_id).unwrap();
        assert_eq!(retrieved.len(), 2);
    }
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[tokio::test]
async fn test_timeseries_large_value() {
    let store = TimeSeriesStore::memory().unwrap();

    let large_value = 1e308; // Near max f64
    let point = DataPoint::new(Utc::now().timestamp(), large_value);

    store
        .write("device1", "large", point.clone())
        .await
        .unwrap();

    store.flush().unwrap();

    let results = store
        .query_range(
            "device1",
            "large",
            point.timestamp - 1,
            point.timestamp + 1,
            None,
        )
        .await
        .unwrap();

    assert_eq!(results.points.len(), 1);
    assert_eq!(results.points[0].value, large_value);
}

#[tokio::test]
async fn test_timeseries_negative_values() {
    let store = TimeSeriesStore::memory().unwrap();

    let point = DataPoint::new(Utc::now().timestamp(), -100.5);

    store
        .write("device1", "negative", point.clone())
        .await
        .unwrap();

    store.flush().unwrap();

    let results = store
        .query_range(
            "device1",
            "negative",
            point.timestamp - 1,
            point.timestamp + 1,
            None,
        )
        .await
        .unwrap();

    assert_eq!(results.points.len(), 1);
    assert_eq!(results.points[0].value, -100.5);
}

#[test]
fn test_session_unicode_content() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("unicode_session.redb");
    let store = SessionStore::open(&db_path).unwrap();

    let session_id = "unicode-session";
    let messages = vec![
        SessionMessage::user("你好世界 🌍"),
        SessionMessage::assistant("Hello World! 你好！"),
    ];

    store.save_session_id(session_id).unwrap();
    store.save_history(session_id, &messages).unwrap();

    let retrieved = store.load_history(session_id).unwrap();
    assert_eq!(retrieved.len(), 2);
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_timeseries_concurrent_writes() {
    let store = TimeSeriesStore::memory().unwrap();
    let store = std::sync::Arc::new(store);

    let mut handles = vec![];

    for i in 0..10 {
        let store = store.clone();
        let handle = tokio::spawn(async move {
            let point = DataPoint::new(Utc::now().timestamp(), i as f64);
            store
                .write(&format!("device{}", i), "counter", point)
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    store.flush().unwrap();

    // Verify all writes succeeded
    for i in 0..10 {
        let results = store
            .query_range(
                &format!("device{}", i),
                "counter",
                0,
                Utc::now().timestamp() + 1,
                None,
            )
            .await
            .unwrap();
        assert_eq!(results.points.len(), 1);
    }
}

#[tokio::test]
async fn test_vector_concurrent_inserts() {
    let store = VectorStore::new();
    let store = std::sync::Arc::new(store);

    let mut handles = vec![];

    for i in 0..10 {
        let store = store.clone();
        let handle = tokio::spawn(async move {
            let embedding: Vec<f32> = (0..4).map(|j| (i * 4 + j) as f32 / 100.0).collect();
            let doc = VectorDocument::new(format!("doc{}", i), embedding);
            store.insert(doc).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all inserts succeeded
    let query = vec![0.0, 0.0, 0.0, 0.0];
    let results = store.search(&query, 20).await.unwrap();
    assert_eq!(results.len(), 10);
}
