"""Aggregate scores.jsonl -> grade-card.md.

Ported from crates/eval-runner/src/report.rs. Weights renormalize over the
`applies[]` subset per case.
"""
from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

WEIGHTS = {
    "tool_accuracy": 25.0,
    "task_completion": 25.0,
    "response_quality": 20.0,
    "context_retention": 15.0,
    "error_recovery": 15.0,
    "language_adherence": 5.0,
}

ERROR_STATUSES = {
    "agent_error",
    "runtime_error",
    "seed_failure",
    "llm_config_error",
    "agent_timeout",
}


def grade_letter(score: float) -> str:
    if score >= 85:
        return "A"
    if score >= 70:
        return "B"
    if score >= 55:
        return "C"
    if score >= 40:
        return "D"
    return "F"


def aggregate(scores_jsonl: str) -> dict:
    agg = {
        "total_cases": 0,
        "malformed": 0,
        "agent_errors": 0,
        "suspected_fallback": 0,
        "by_dimension": defaultdict(list),
        "by_lang": defaultdict(list),
        "overall_per_case": [],
    }

    for line in (scores_jsonl or "").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            s = json.loads(line)
        except json.JSONDecodeError:
            agg["malformed"] += 1
            continue

        agg["total_cases"] += 1
        if s.get("suspected_fallback"):
            agg["suspected_fallback"] += 1
        if s.get("status") in ERROR_STATUSES:
            agg["agent_errors"] += 1

        scores = s.get("scores") or {}
        if not isinstance(scores, dict):
            continue
        present = {k: v for k, v in scores.items() if isinstance(v, (int, float))}
        if not present:
            continue
        total_w = sum(WEIGHTS.get(k, 0.0) for k in present)
        case_overall = 0.0
        for k, v in present.items():
            agg["by_dimension"][k].append(float(v))
            if total_w > 0:
                case_overall += float(v) * WEIGHTS.get(k, 0.0) / total_w
        # case_overall is 0-10; convert to 0-100.
        agg["overall_per_case"].append(case_overall * 10.0)
        agg["by_lang"][s.get("lang", "?")].append(case_overall * 10.0)

    return agg


def overall(agg: dict) -> float:
    v = agg["overall_per_case"]
    return sum(v) / len(v) if v else 0.0


def write_grade_card(agg: dict, out_path: Path):
    grade = grade_letter(overall(agg))
    md = ["# HeraMind Chat Eval Report", ""]
    md.append(f"## Overall Grade: **{grade} ({overall(agg):.1f})**")
    md.append("")

    denom = agg["total_cases"] + agg["malformed"]
    malformed_rate = agg["malformed"] / denom if denom else 0.0
    if malformed_rate > 0.05:
        md.append(
            f"⚠️ Malformed score lines: {malformed_rate*100:.1f}% — "
            "results may be unreliable."
        )
        md.append("")
    if agg["agent_errors"] > 0:
        md.append(
            f"⚠️ Agent failures: {agg['agent_errors']} case(s) excluded from averages."
        )
        md.append("")

    md.append("| Dimension | Avg (0-10) |")
    md.append("|---|---|")
    dim_avg = {
        k: sum(v) / len(v) for k, v in agg["by_dimension"].items() if v
    }
    for dim, avg in dim_avg.items():
        md.append(f"| {dim} | {avg:.2f} |")

    md.append("")
    md.append("## By Language")
    md.append("")
    md.append("| Lang | Cases | Avg (0-100) |")
    md.append("|---|---|---|")
    for lang, vs in agg["by_lang"].items():
        avg = sum(vs) / len(vs) if vs else 0.0
        md.append(f"| {lang} | {len(vs)} | {avg:.1f} |")

    md.append("")
    md.append(f"Suspected fallback cases: {agg['suspected_fallback']}")
    md.append("")

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(md))
