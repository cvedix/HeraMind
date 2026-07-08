"""3-condition suspected_fallback detection (spec §7).

Ported from crates/eval-runner/src/fallback.rs. A "suspected fallback" is the
case where the agent's LLM never actually ran — the session silently fell back
to keyword matching. Detect by: ALL turns empty+fast AND expectations mention
a tool word.
"""
from __future__ import annotations

TOOL_WORDS = (
    "调用", "invoke", "call", "execute", "执行", "运行",
    "create", "创建", "delete", "删除", "update", "更新",
    "device", "rule", "agent", "dashboard", "transform",
    "extension", "channel", "push",
)


def detect_suspected_fallback(turns: list[dict], expectations: list[str]) -> bool:
    # Condition 1+2: ALL turns have empty tool_calls AND < 500ms.
    if not turns:
        return False
    for t in turns:
        if t.get("tool_calls"):
            return False
        if t.get("processing_time_ms", 0) >= 500:
            return False
    # Condition 3: at least one expectation mentions a tool word.
    for e in expectations:
        lower = e.lower()
        if any(w.lower() in lower for w in TOOL_WORDS):
            return True
    return False
