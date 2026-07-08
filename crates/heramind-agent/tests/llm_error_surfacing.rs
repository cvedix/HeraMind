//! Integration tests for agent LLM error surfacing.
//!
//! Verifies that LLM failures propagate as Failed execution records
//! instead of silently degrading to rule-based analysis.
//!
//! Note: These tests cover LlmError::is_permanent() classification.
//! End-to-end agent-execution verification is done via the manual
//! verification step (Task 8) because it requires a full AgentExecutor
//! bootstrap with mock stores and a mock LlmRuntime.

use heramind_core::llm::backend::LlmError;

#[test]
fn permanent_llm_error_classified_correctly() {
    // 4xx (except 429) are permanent — require user action.
    let quota_err = LlmError::Api {
        status: 403,
        body: "{\"error\":{\"message\":\"free tier exhausted\",\"type\":\"AllocationQuota\"}}"
            .into(),
    };
    assert!(quota_err.is_permanent(), "403 should be permanent");

    assert!(LlmError::Api {
        status: 400,
        body: "".into()
    }
    .is_permanent());
    assert!(LlmError::Api {
        status: 401,
        body: "".into()
    }
    .is_permanent());
    assert!(LlmError::Api {
        status: 404,
        body: "".into()
    }
    .is_permanent());
}

#[test]
fn transient_llm_error_classified_correctly() {
    // 5xx, 429, and non-HTTP variants are transient — may succeed on retry.
    let timeout_err = LlmError::Timeout(60);
    assert!(!timeout_err.is_permanent(), "timeout should be transient");

    assert!(!LlmError::Api {
        status: 429,
        body: "rate limited".into()
    }
    .is_permanent());
    assert!(!LlmError::Api {
        status: 500,
        body: "".into()
    }
    .is_permanent());
    assert!(!LlmError::Api {
        status: 503,
        body: "".into()
    }
    .is_permanent());
    assert!(!LlmError::Network("connection refused".into()).is_permanent());
}

#[test]
fn api_variant_display_format() {
    // Display output must match the prior Generation(format!(...)) format
    // so log/error consumers see no difference.
    let e = LlmError::Api {
        status: 403,
        body: "quota exhausted".into(),
    };
    let s = format!("{}", e);
    assert!(
        s.contains("403"),
        "Display should include status: got {}",
        s
    );
    assert!(
        s.contains("quota exhausted"),
        "Display should include body: got {}",
        s
    );
}
