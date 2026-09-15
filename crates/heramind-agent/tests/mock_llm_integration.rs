//! Proves `MockLlmRuntime` (under the `test-utils` feature) is reachable from
//! an integration test — the gap the old `#[cfg(test)]` `MockStreamRuntime`
//! could not fill (it was crate-local to `heramind-core`, invisible to these
//! separate-crate tests). Run with: `cargo test -p heramind-agent --features
//! test-utils`.

#![cfg(feature = "test-utils")]

use heramind_agent::testing_helpers::mock_llm::{MockLlmRuntime, MockResponse};
use heramind_core::llm::backend::{LlmInput, LlmRuntime};

#[tokio::test]
async fn integration_test_drives_mock_across_rounds() {
    // Script a 2-round ReAct: round 1 calls a tool, round 2 returns final text.
    let rt = MockLlmRuntime::new(vec![
        MockResponse::tool_call(
            "shell",
            serde_json::json!({ "command": "heramind device list" }),
        ),
        MockResponse::text("Done: found 3 devices"),
    ]);

    let o1 = rt.generate(LlmInput::new("list devices")).await.unwrap();
    assert!(
        o1.tool_calls.is_some(),
        "round 1 should carry structured tool_calls for the loop to read"
    );

    let o2 = rt.generate(LlmInput::new("list devices")).await.unwrap();
    assert_eq!(o2.text, "Done: found 3 devices");
    assert!(
        o2.tool_calls.is_none(),
        "round 2 is a natural text completion"
    );

    assert_eq!(rt.call_count(), 2);
}

#[tokio::test]
async fn integration_test_default_response_after_script_exhausted() {
    // Drives a fixed topology: the loop keeps calling past the script; the mock
    // returns a deterministic default so tests don't panic on over-call.
    let rt = MockLlmRuntime::with_default(
        vec![MockResponse::text("first")],
        MockResponse::text("repeat"),
    );
    let _ = rt.generate(LlmInput::new("x")).await.unwrap();
    for _ in 0..5 {
        let o = rt.generate(LlmInput::new("x")).await.unwrap();
        assert_eq!(o.text, "repeat");
    }
    assert_eq!(rt.call_count(), 6);
}
