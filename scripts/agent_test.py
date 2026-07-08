#!/usr/bin/env python3
"""
HeraMind Agent System - Tool Calling & Context Quality Test Suite

Tests:
  T1: Basic single-tool calling (list/get/query)
  T2: Multi-step sequential tool calling (create→update→verify)
  T3: Context retention across 20+ turns
  T4: Multi-tool coordination (device+rule+agent)
  T5: Error recovery & fallback
  T6: Chinese/English mixed instructions

Usage:
  python3 scripts/agent_test.py [--api http://localhost:9375/api] --user Admin --pass zxc707cxz
"""

import json
import sys
import time
import argparse
import requests
from dataclasses import dataclass, field
from typing import Optional

# ── Helpers ──────────────────────────────────────────────────────────────────

def login(api_base: str, username: str, password: str) -> str:
    r = requests.post(f"{api_base}/auth/login",
                      json={"username": username, "password": password})
    r.raise_for_status()
    return r.json()["token"]

def create_session(api_base: str, token: str) -> str:
    r = requests.post(f"{api_base}/sessions",
                      headers={"Authorization": f"Bearer {token}"})
    r.raise_for_status()
    return r.json()["data"]["sessionId"]

def chat(api_base: str, token: str, session_id: str, message: str, timeout: int = 120) -> dict:
    r = requests.post(f"{api_base}/sessions/{session_id}/chat",
                      headers={"Authorization": f"Bearer {token}",
                               "Content-Type": "application/json"},
                      json={"message": message},
                      timeout=timeout)
    r.raise_for_status()
    # Use strict=False to handle control characters in thinking content
    return json.loads(r.text, strict=False)

def delete_session(api_base: str, token: str, session_id: str):
    requests.delete(f"{api_base}/sessions/{session_id}",
                    headers={"Authorization": f"Bearer {token}"})

# ── Test Infrastructure ──────────────────────────────────────────────────────

@dataclass
class TurnResult:
    turn: int
    user_msg: str
    assistant_msg: str = ""
    tools_used: list = field(default_factory=list)
    processing_ms: int = 0
    passed: bool = False
    reason: str = ""

@dataclass
class TestResult:
    test_id: str
    test_name: str
    turns: list = field(default_factory=list)
    total_turns: int = 0
    passed_turns: int = 0
    score: float = 0.0

    def finalize(self):
        self.total_turns = len(self.turns)
        self.passed_turns = sum(1 for t in self.turns if t.passed)
        self.score = self.passed_turns / self.total_turns if self.total_turns > 0 else 0.0

def evaluate(response: dict, expect_tools: list = None, expect_keywords: list = None,
             expect_no_tools: bool = False) -> tuple[bool, str]:
    """Evaluate a single turn's response."""
    tools = response.get("toolsUsed", [])
    content = response.get("response", "")

    # Check tool expectations
    if expect_no_tools and tools:
        return False, f"Expected no tools but got: {tools}"
    if expect_tools:
        # expect_tools: at least one must be present (OR semantics for aliases)
        if not any(t in tools for t in expect_tools):
            return False, f"Expected one of {expect_tools} but got {tools}"

    # Check keyword expectations
    if expect_keywords:
        content_lower = content.lower()
        for kw in expect_keywords:
            if kw.lower() not in content_lower:
                return False, f"Expected keyword '{kw}' not found in response"

    return True, "OK"

def run_turn(api_base: str, token: str, session_id: str, turn_num: int,
             message: str, expect_tools=None, expect_keywords=None,
             expect_no_tools=False) -> TurnResult:
    """Run a single conversation turn and evaluate."""
    result = TurnResult(turn=turn_num, user_msg=message)

    try:
        resp = chat(api_base, token, session_id, message)
        result.assistant_msg = resp.get("response", "")
        result.tools_used = resp.get("toolsUsed", [])
        result.processing_ms = resp.get("processingTimeMs", 0)

        passed, reason = evaluate(resp, expect_tools, expect_keywords, expect_no_tools)
        result.passed = passed
        result.reason = reason

    except Exception as e:
        result.passed = False
        result.reason = f"Error: {e}"

    status = "✓" if result.passed else "✗"
    tools_str = ",".join(result.tools_used) if result.tools_used else "none"
    print(f"  T{turn_num:02d} {status} | tools=[{tools_str}] {result.processing_ms}ms | {result.reason}")
    if not result.passed:
        preview = result.assistant_msg[:100].replace("\n", " ")
        print(f"       Response: {preview}...")

    return result


# ── Test Cases ────────────────────────────────────────────────────────────────

def test_T1_basic_tool_calling(api_base: str, token: str) -> TestResult:
    """T1: Basic single-tool calling — verify the LLM correctly invokes
    individual tools for list, get, and query operations."""
    tr = TestResult("T1", "Basic Single-Tool Calling")
    sid = create_session(api_base, token)
    print(f"\n{'='*60}\nT1: Basic Single-Tool Calling (session: {sid[:8]}...)\n{'='*60}")

    turns = [
        ("你好，请介绍一下你自己", [], None, True),
        ("列出所有设备", ["device"], None, False),
        ("列出所有规则", ["rule"], None, False),
        ("列出所有 agent", ["agent"], None, False),
        ("查看未读消息", ["message"], None, False),
        ("列出所有告警", ["message", "alert"], None, False),  # alert is an alias for message tool
        ("列出所有扩展", ["extension"], None, False),
        ("获取系统状态", None, ["运行","状态","系统"], False),
    ]

    for i, (msg, tools, keywords, no_tools) in enumerate(turns, 1):
        tr.turns.append(run_turn(api_base, token, sid, i, msg,
                                 expect_tools=tools, expect_keywords=keywords,
                                 expect_no_tools=no_tools))

    delete_session(api_base, token, sid)
    tr.finalize()
    return tr


def test_T2_multi_step_sequential(api_base: str, token: str) -> TestResult:
    """T2: Multi-step sequential tool calling — create → update → verify.
    Tests that context is maintained across related operations."""
    tr = TestResult("T2", "Multi-Step Sequential Tool Calling")
    sid = create_session(api_base, token)
    print(f"\n{'='*60}\nT2: Multi-Step Sequential (session: {sid[:8]}...)\n{'='*60}")

    # Phase 1: Create agent
    tr.turns.append(run_turn(api_base, token, sid, 1,
        "创建一个测试用的 agent，名称叫 test-monitor，用事件触发，提示词是：每5分钟检查一次温度",
        expect_tools=["agent"]))

    # Phase 2: Verify creation
    tr.turns.append(run_turn(api_base, token, sid, 2,
        "查看 test-monitor 这个 agent 的详细信息",
        expect_tools=["agent"]))

    # Phase 3: Update prompt
    tr.turns.append(run_turn(api_base, token, sid, 3,
        "把 test-monitor 的提示词改成英文：Check temperature every 5 minutes and alert if above 30C",
        expect_tools=["agent"]))

    # Phase 4: Verify update
    tr.turns.append(run_turn(api_base, token, sid, 4,
        "确认 test-monitor 的提示词已经更新为英文了",
        expect_tools=["agent"],
        expect_keywords=["check", "temperature"]))

    # Phase 5: Pause agent
    tr.turns.append(run_turn(api_base, token, sid, 5,
        "暂停 test-monitor",
        expect_tools=["agent"]))

    # Phase 6: Verify paused
    tr.turns.append(run_turn(api_base, token, sid, 6,
        "test-monitor 现在是什么状态？",
        expect_tools=["agent"],
        expect_keywords=["暂停", "paused"]))

    # Phase 7: Resume agent
    tr.turns.append(run_turn(api_base, token, sid, 7,
        "恢复 test-monitor",
        expect_tools=["agent"]))

    # Phase 8: Send message
    tr.turns.append(run_turn(api_base, token, sid, 8,
        "给 test-monitor 发送一条消息：请注意湿度变化",
        expect_tools=["agent"]))

    # Phase 9: View executions
    tr.turns.append(run_turn(api_base, token, sid, 9,
        "查看 test-monitor 的执行统计",
        expect_tools=["agent"]))

    # Phase 10: Delete agent
    # Note: agent tool doesn't have delete, so we test list after operations
    tr.turns.append(run_turn(api_base, token, sid, 10,
        "列出所有 agent，看看 test-monitor 是否在列表中",
        expect_tools=["agent"],
        expect_keywords=["test-monitor"]))

    delete_session(api_base, token, sid)
    tr.finalize()
    return tr


def test_T3_context_retention(api_base: str, token: str) -> TestResult:
    """T3: Context retention — 20+ turn conversation testing that
    the LLM remembers earlier context without re-querying."""
    tr = TestResult("T3", "Context Retention (20+ turns)")
    sid = create_session(api_base, token)
    print(f"\n{'='*60}\nT3: Context Retention (session: {sid[:8]}...)\n{'='*60}")

    # Build shared context first
    tr.turns.append(run_turn(api_base, token, sid, 1,
        "列出所有 agent", expect_tools=["agent"]))

    tr.turns.append(run_turn(api_base, token, sid, 2,
        "列出所有设备", expect_tools=["device"]))

    tr.turns.append(run_turn(api_base, token, sid, 3,
        "列出所有规则", expect_tools=["rule"]))

    # Filler turns to push context
    tr.turns.append(run_turn(api_base, token, sid, 4,
        "今天天气怎么样？", expect_no_tools=True))

    tr.turns.append(run_turn(api_base, token, sid, 5,
        "你能做什么？", expect_no_tools=True))

    # Now test if it remembers the first agent
    tr.turns.append(run_turn(api_base, token, sid, 6,
        "刚才你看到的第一个 agent 叫什么名字？",
        expect_keywords=["people", "analysis", "agent"]))

    # More filler
    tr.turns.append(run_turn(api_base, token, sid, 7,
        "帮我查一下有没有设备在线", expect_tools=["device"]))

    tr.turns.append(run_turn(api_base, token, sid, 8,
        "查看所有未读消息", expect_tools=["message"]))

    tr.turns.append(run_turn(api_base, token, sid, 9,
        "有哪些扩展可以安装？", expect_tools=["extension"]))

    tr.turns.append(run_turn(api_base, token, sid, 10,
        "告诉我一个有趣的事实", expect_no_tools=True))

    # Test context: does it remember rule count?
    tr.turns.append(run_turn(api_base, token, sid, 11,
        "之前你列出了多少条规则？",
        # Don't require tools - should answer from context
        ))

    # More turns
    tr.turns.append(run_turn(api_base, token, sid, 12,
        "列出所有告警", expect_tools=["message", "alert"]))

    tr.turns.append(run_turn(api_base, token, sid, 13,
        "查看系统状态", ))

    tr.turns.append(run_turn(api_base, token, sid, 14,
        "什么是 IoT 物联网？", expect_no_tools=True))

    tr.turns.append(run_turn(api_base, token, sid, 15,
        "我之前让你查过什么设备？还记得吗？",
        # Should reference earlier device query from context
        ))

    # More context tests
    tr.turns.append(run_turn(api_base, token, sid, 16,
        "更新第一个 agent 的提示词，加上：每10分钟检查一次",
        expect_tools=["agent"]))

    tr.turns.append(run_turn(api_base, token, sid, 17,
        "刚才更新的 agent 现在的提示词是什么？",
        expect_tools=["agent"],
        expect_keywords=["10"]))

    tr.turns.append(run_turn(api_base, token, sid, 18,
        "帮我创建一条规则：当温度超过30度时通知我",
        expect_tools=["rule"]))

    tr.turns.append(run_turn(api_base, token, sid, 19,
        "刚才创建的规则叫什么？触发条件是什么？",
        # Should recall from context
        ))

    tr.turns.append(run_turn(api_base, token, sid, 20,
        "总结一下我们这次对话中你做了哪些操作",
        # Should list all operations from context
        ))

    delete_session(api_base, token, sid)
    tr.finalize()
    return tr


def test_T4_multi_tool_coordination(api_base: str, token: str) -> TestResult:
    """T4: Multi-tool coordination — combining device + rule + agent
    in a single complex request."""
    tr = TestResult("T4", "Multi-Tool Coordination")
    sid = create_session(api_base, token)
    print(f"\n{'='*60}\nT4: Multi-Tool Coordination (session: {sid[:8]}...)\n{'='*60}")

    # Complex request requiring multiple tools
    tr.turns.append(run_turn(api_base, token, sid, 1,
        "先查看所有设备，然后查看所有规则，最后列出所有 agent，给我一个整体报告",
        # Should use multiple tools
        ))

    tr.turns.append(run_turn(api_base, token, sid, 2,
        "根据刚才看到的信息，哪些设备还没有对应的监控规则？",
        # Should cross-reference device and rule data
        ))

    tr.turns.append(run_turn(api_base, token, sid, 3,
        "帮我为每个没有监控规则的设备创建一条温度报警规则，温度阈值35度",
        expect_tools=["rule"]))

    tr.turns.append(run_turn(api_base, token, sid, 4,
        "现在列出所有规则，确认刚才创建的规则都在",
        expect_tools=["rule"]))

    tr.turns.append(run_turn(api_base, token, sid, 5,
        "为这些规则分别创建一个 agent 来执行，名称统一用 auto-monitor-设备名",
        expect_tools=["agent"]))

    tr.turns.append(run_turn(api_base, token, sid, 6,
        "查看所有 agent 的状态，确认新创建的都在运行",
        expect_tools=["agent"]))

    tr.turns.append(run_turn(api_base, token, sid, 7,
        "给所有新创建的 agent 发送消息：注意观察湿度数据",
        expect_tools=["agent"]))

    # Now test if it remembers everything
    tr.turns.append(run_turn(api_base, token, sid, 8,
        "总结一下刚才我们一共创建了多少条规则和多少个 agent？分别叫什么？",
        ))

    tr.turns.append(run_turn(api_base, token, sid, 9,
        "查看未读消息，看看有没有新通知",
        expect_tools=["message"]))

    tr.turns.append(run_turn(api_base, token, sid, 10,
        "检查系统健康状态",
        ))

    delete_session(api_base, token, sid)
    tr.finalize()
    return tr


def test_T5_error_recovery(api_base: str, token: str) -> TestResult:
    """T5: Error recovery — test behavior with invalid inputs,
    non-existent resources, and ambiguous requests."""
    tr = TestResult("T5", "Error Recovery & Fallback")
    sid = create_session(api_base, token)
    print(f"\n{'='*60}\nT5: Error Recovery (session: {sid[:8]}...)\n{'='*60}")

    # Invalid ID
    tr.turns.append(run_turn(api_base, token, sid, 1,
        "查看 agent 不存在的agent123 的详细信息", ))

    # Ambiguous request
    tr.turns.append(run_turn(api_base, token, sid, 2,
        "帮我管理一下", ))

    # Non-existent device
    tr.turns.append(run_turn(api_base, token, sid, 3,
        "控制设备 nonexistent-device，把灯关了", ))

    # Invalid rule operation
    tr.turns.append(run_turn(api_base, token, sid, 4,
        "删除规则 fake-rule-id", ))

    # Multiple errors followed by valid request
    tr.turns.append(run_turn(api_base, token, sid, 5,
        "更新 agent ghost-agent 的提示词为 hello",
        expect_tools=["agent"]))

    # Should still work after errors
    tr.turns.append(run_turn(api_base, token, sid, 6,
        "列出所有 agent，看看有哪些是正常的",
        expect_tools=["agent"]))

    # Conflicting instructions
    tr.turns.append(run_turn(api_base, token, sid, 7,
        "暂停 test-monitor 同时也恢复它", ))

    # Very long message
    tr.turns.append(run_turn(api_base, token, sid, 8,
        "我有一个很长的需求：" + "这是一个测试。" * 50 + "请总结一下", ))

    # Empty intent
    tr.turns.append(run_turn(api_base, token, sid, 9,
        "嗯", ))

    # Recovery
    tr.turns.append(run_turn(api_base, token, sid, 10,
        "好了，现在回到正题，列出所有设备的状态",
        expect_tools=["device"]))

    delete_session(api_base, token, sid)
    tr.finalize()
    return tr


def test_T6_bilingual(api_base: str, token: str) -> TestResult:
    """T6: Chinese/English mixed — test language switching
    and bilingual tool parameter handling."""
    tr = TestResult("T6", "Chinese/English Mixed Instructions")
    sid = create_session(api_base, token)
    print(f"\n{'='*60}\nT6: Bilingual (session: {sid[:8]}...)\n{'='*60}")

    tr.turns.append(run_turn(api_base, token, sid, 1,
        "List all agents", expect_tools=["agent"]))

    tr.turns.append(run_turn(api_base, token, sid, 2,
        "用中文回答，刚才有多少个 agent？", ))

    tr.turns.append(run_turn(api_base, token, sid, 3,
        "Update the first agent's prompt to: Monitor temperature and humidity sensors",
        expect_tools=["agent"]))

    tr.turns.append(run_turn(api_base, token, sid, 4,
        "确认提示词已经更新了", expect_tools=["agent"]))

    tr.turns.append(run_turn(api_base, token, sid, 5,
        "Show me all devices", expect_tools=["device"]))

    tr.turns.append(run_turn(api_base, token, sid, 6,
        "帮我看看这些设备哪些是在线的", ))

    tr.turns.append(run_turn(api_base, token, sid, 7,
        "Create a new rule: when temperature > 35, send alert",
        expect_tools=["rule"]))

    tr.turns.append(run_turn(api_base, token, sid, 8,
        "这条规则创建成功了吗？查看它的详细信息", expect_tools=["rule"]))

    tr.turns.append(run_turn(api_base, token, sid, 9,
        "现在把这条规则的温度阈值改成40度",
        expect_tools=["rule"]))

    tr.turns.append(run_turn(api_base, token, sid, 10,
        "Summarize everything we did in this conversation, in English",
        ))

    delete_session(api_base, token, sid)
    tr.finalize()
    return tr


# ── Main Runner ──────────────────────────────────────────────────────────────

def print_summary(results: list[TestResult]):
    print(f"\n{'='*70}")
    print(f"  TEST SUMMARY")
    print(f"{'='*70}")
    print(f"{'Test':<6} {'Name':<35} {'Score':<8} {'Pass/Total':<12} {'Status'}")
    print(f"{'-'*70}")

    total_pass = 0
    total_all = 0

    for r in results:
        status = "PASS" if r.score >= 0.7 else "WARN" if r.score >= 0.5 else "FAIL"
        icon = {"PASS": "✅", "WARN": "⚠️", "FAIL": "❌"}[status]
        print(f"{r.test_id:<6} {r.test_name:<35} {r.score:>5.0%}   {r.passed_turns:>3}/{r.total_turns:<6} {icon} {status}")
        total_pass += r.passed_turns
        total_all += r.total_turns

    print(f"{'-'*70}")
    overall = total_pass / total_all if total_all > 0 else 0
    print(f"{'TOTAL':<6} {'':35} {overall:>5.0%}   {total_pass:>3}/{total_all:<6}")

    print(f"\n{'='*70}")
    print("  OPTIMIZATION DIRECTIONS")
    print(f"{'='*70}")

    # Analysis
    categories = {}
    for r in results:
        for t in r.turns:
            if not t.passed:
                key = r.test_id
                if key not in categories:
                    categories[key] = []
                categories[key].append(t)

    if not categories:
        print("  All tests passed! No immediate optimization needed.")
    else:
        for test_id, failures in categories.items():
            print(f"\n  {test_id} failures ({len(failures)} turns):")
            for f in failures:
                print(f"    - T{f.turn:02d}: {f.user_msg[:40]}... → {f.reason}")


def main():
    parser = argparse.ArgumentParser(description="HeraMind Agent Test Suite")
    parser.add_argument("--api", default="http://localhost:9375/api")
    parser.add_argument("--user", default="Admin")
    parser.add_argument("--password", required=True)
    parser.add_argument("--tests", default="all",
                       help="Comma-separated test IDs (T1,T2,...) or 'all'")
    args = parser.parse_args()

    print("HeraMind Agent System — Tool Calling & Context Quality Test")
    print(f"API: {args.api}")

    # Login
    token = login(args.api, args.user, args.password)
    print(f"Logged in as {args.user}")

    # Select tests
    all_tests = {
        "T1": test_T1_basic_tool_calling,
        "T2": test_T2_multi_step_sequential,
        "T3": test_T3_context_retention,
        "T4": test_T4_multi_tool_coordination,
        "T5": test_T5_error_recovery,
        "T6": test_T6_bilingual,
    }

    if args.tests == "all":
        selected = list(all_tests.keys())
    else:
        selected = [t.strip().upper() for t in args.tests.split(",")]

    results = []
    start = time.time()

    for test_id in selected:
        if test_id not in all_tests:
            print(f"Unknown test: {test_id}, skipping")
            continue
        try:
            r = all_tests[test_id](args.api, token)
            results.append(r)
        except Exception as e:
            print(f"\n  TEST ABORTED: {e}")
            tr = TestResult(test_id, f"{test_id} (aborted)")
            tr.finalize()
            results.append(tr)

    elapsed = time.time() - start
    print_summary(results)
    print(f"\nTotal time: {elapsed:.1f}s")


if __name__ == "__main__":
    main()
