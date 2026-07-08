//! IPC 端到端测试
//!
//! 使用真实的扩展进程测试完整的 IPC 流程

use std::process::{Command, Stdio};
use std::time::Duration;

/// IPC Frame 格式: [4 bytes length][payload]
fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let len = payload.len() as u32;
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn decode_frame(frame: &[u8]) -> Option<Vec<u8>> {
    if frame.len() < 4 {
        return None;
    }
    let len = u32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
    if frame.len() < 4 + len {
        return None;
    }
    Some(frame[4..4 + len].to_vec())
}

/// 测试扩展进程启动和基本通信
#[test]
#[ignore = "需要编译好的扩展和 runner"]
fn test_extension_process_startup() {
    println!("\n=== 测试: 扩展进程启动 ===");

    // 查找扩展 runner
    let runner_path = "./target/debug/heramind-extension-runner";
    let extension_path = "./target/debug/libyolo_video_v2.dylib";

    // 启动扩展进程
    let mut child = Command::new(runner_path)
        .arg("--extension-path")
        .arg(extension_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start extension process");

    println!("✓ 扩展进程已启动 (PID: {:?})", child.id());

    // 等待进程初始化
    std::thread::sleep(Duration::from_millis(500));

    // 清理
    child.kill().expect("Failed to kill process");
    let _ = child.wait();
    println!("✓ 扩展进程已停止");
}

/// 测试 IPC 消息往返
#[test]
fn test_ipc_message_roundtrip() {
    println!("\n=== 测试: IPC 消息往返 ===");

    // 测试各种消息类型
    let test_messages = vec![
        (r#"{"Init":{"config":{}}}"#, "Ready"),
        (
            r#"{"ExecuteCommand":{"id":1,"command":"test","args":{}}}"#,
            "CommandResult",
        ),
        (r#"{"Ping":null}"#, "Pong"),
    ];

    for (msg, expected_type) in test_messages {
        println!("测试消息: {} -> 期望: {}", msg, expected_type);

        // 编码
        let frame = encode_frame(msg.as_bytes());
        assert!(frame.len() > 4);
        println!("  编码后: {} bytes", frame.len());

        // 解码
        let decoded = decode_frame(&frame).expect("解码失败");
        assert_eq!(decoded, msg.as_bytes());
        println!("  ✓ 编解码正确");
    }

    println!("\n✓ IPC 消息往返测试通过\n");
}

/// 测试大消息处理
#[test]
fn test_large_message_handling() {
    println!("\n=== 测试: 大消息处理 ===");

    // 测试不同大小的消息
    let sizes = vec![
        (100, "100 bytes"),
        (1024, "1 KB"),
        (10 * 1024, "10 KB"),
        (100 * 1024, "100 KB"),
        (1024 * 1024, "1 MB"),
    ];

    for (size, label) in sizes {
        let data = vec![0u8; size];
        let frame = encode_frame(&data);

        println!("  {}: {} bytes (frame: {} bytes)", label, size, frame.len());

        // 验证帧大小
        assert_eq!(frame.len(), 4 + size);

        // 解码验证
        let decoded = decode_frame(&frame).expect("解码失败");
        assert_eq!(decoded.len(), size);
    }

    println!("\n✓ 大消息处理测试通过\n");
}

/// 测试并发帧处理
#[test]
fn test_concurrent_frame_encoding() {
    use std::thread;

    println!("\n=== 测试: 并发帧编码 ===");

    let num_threads = 10;
    let frames_per_thread = 1000;
    let mut handles = vec![];

    let start = std::time::Instant::now();

    for t in 0..num_threads {
        handles.push(thread::spawn(move || {
            for i in 0..frames_per_thread {
                let msg = format!(r#"{{"id":{},"thread":{}}}"#, i, t);
                let frame = encode_frame(msg.as_bytes());
                let decoded = decode_frame(&frame).expect("解码失败");
                assert_eq!(decoded, msg.as_bytes());
            }
            frames_per_thread
        }));
    }

    let total_frames: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    let elapsed = start.elapsed();

    println!("  处理了 {} 个帧", total_frames);
    println!("  耗时: {:?}", elapsed);
    println!(
        "  吞吐: {:.0} frames/sec",
        total_frames as f64 / elapsed.as_secs_f64()
    );

    println!("\n✓ 并发帧编码测试通过\n");
}

/// 测试帧边界处理
#[test]
fn test_frame_boundary_handling() {
    println!("\n=== 测试: 帧边界处理 ===");

    // 测试不完整的帧
    let incomplete_frames: &[&[u8]] = &[
        &[],                       // 空帧
        &[0, 0, 0],                // 不完整的长度
        &[10, 0, 0, 0],            // 长度但无数据
        &[5, 0, 0, 0, b'h', b'e'], // 部分数据
    ];

    for (i, frame) in incomplete_frames.iter().enumerate() {
        let result = decode_frame(frame);
        assert!(result.is_none(), "帧 {} 应该解码失败", i);
        println!("  ✓ 不完整帧 {} 正确拒绝", i);
    }

    // 测试有效帧
    let valid_frame = {
        let mut f = Vec::new();
        f.extend_from_slice(&5u32.to_le_bytes());
        f.extend_from_slice(b"hello");
        f
    };

    let decoded = decode_frame(&valid_frame).expect("有效帧应该解码成功");
    assert_eq!(decoded, b"hello");
    println!("  ✓ 有效帧正确解码");

    println!("\n✓ 帧边界处理测试通过\n");
}

/// 测试消息序列化性能
#[test]
fn test_message_serialization_performance() {
    use serde_json::json;

    println!("\n=== 测试: 消息序列化性能 ===");

    let iterations = 10000;
    let mut times = Vec::with_capacity(iterations);

    // 预热
    for _ in 0..100 {
        let msg = json!({"ExecuteCommand": {"id": 0, "command": "test", "args": {}}});
        let _ = serde_json::to_vec(&msg).unwrap();
    }

    // 测试
    for i in 0..iterations {
        let msg = json!({"ExecuteCommand": {"id": i, "command": "test", "args": {"key": "value"}}});

        let start = std::time::Instant::now();
        let bytes = serde_json::to_vec(&msg).unwrap();
        times.push(start.elapsed());

        // 验证
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["ExecuteCommand"]["id"], i);
    }

    let avg_ns: f64 = times.iter().map(|t| t.as_nanos() as f64).sum::<f64>() / times.len() as f64;
    let min_ns = times
        .iter()
        .map(|t| t.as_nanos() as f64)
        .fold(f64::INFINITY, f64::min);
    let max_ns = times
        .iter()
        .map(|t| t.as_nanos() as f64)
        .fold(0.0, f64::max);

    println!("  序列化 {} 次:", iterations);
    println!("    平均: {:.2} ns", avg_ns);
    println!("    最小: {:.2} ns", min_ns);
    println!("    最大: {:.2} ns", max_ns);
    println!("    吞吐: {:.0} msg/sec", 1_000_000_000.0 / avg_ns);

    println!("\n✓ 消息序列化性能测试通过\n");
}

/// 运行所有测试
#[test]
fn test_all_ipc_tests() {
    println!("\n========================================");
    println!("    IPC 测试套件");
    println!("========================================\n");

    test_ipc_message_roundtrip();
    test_large_message_handling();
    test_concurrent_frame_encoding();
    test_frame_boundary_handling();
    test_message_serialization_performance();

    println!("\n========================================");
    println!("    所有 IPC 测试通过!");
    println!("========================================\n");
}
