#!/usr/bin/env python3
"""Skill ↔ CLI drift checker.

Scans builtin skill `.md` files for `heramind <domain> <action> ...` references
in fenced ```bash blocks, then emits a JSON manifest consumed by the
`skill_cli_drift` Rust integration test. The Rust side reflects the clap
surface via `Args::try_parse_from` — a parse failure means the command no
longer exists in the CLI.

Usage:
    python scripts/check_skill_cli_drift.py --out /tmp/skill_cli_manifest.json
    python scripts/check_skill_cli_drift.py --pretty   # stdout, human review

Why two-step (Python extract + Rust reflect) instead of one shell script:
- Python is faster and more reliable for fenced-code extraction.
- Rust clap reflection is the only authoritative source of the current CLI
  surface; --help text parsing is brittle.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from collections import defaultdict


SKILL_GLOB = "*.md"
SKILLS_DIR = Path(__file__).resolve().parent.parent / "crates" / "heramind-agent" / "src" / "skills" / "builtins"

# Match `heramind <domain> <action>` (optionally with subdomain like `device drafts list`).
# Skip:
#   - comment lines starting with #
#   - lines starting with `$ ` (shell prompt convention, not real command)
#   - inline code spans in prose (we only scan fenced blocks)
HERAMIND_RE = re.compile(
    r"""
    (?:^|\s)                  # word boundary
    heramind\s+                # the binary name
    (?P<domain>[a-z][a-z0-9_-]+)  # top-level domain, e.g. agent/rule/device
    (?:
        \s+--?[a-z]           # flag directly after domain (e.g. `heramind device --limit`)
        |
        \s+(?P<action>[a-z][a-z0-9_-]+)  # OR an action word
        (?:
            \s+(?P<sub>[a-z][a-z0-9_-]+)  # optional sub-action (e.g. drafts list)
        )?
    )
    """,
    re.VERBOSE,
)

# Subdomains where the second positional is itself a subcommand group,
# not an instance ID. We capture (domain, action[, sub]) tuples.
SUBDOMAINS = {
    ("device", "drafts"),
    ("device", "types"),
    ("agent", "control"),
}


def extract_commands(md_text: str) -> set[tuple[str, str, str | None]]:
    """Pull `heramind <domain> <action> [<sub>]` tuples from ```bash blocks.

    Returns a set so duplicate occurrences across an example collapse cleanly.
    """
    found: set[tuple[str, str, str | None]] = set()
    in_block = False
    for line in md_text.splitlines():
        stripped = line.strip()
        if stripped.startswith("```"):
            in_block = stripped.removeprefix("```").strip().lower() in {"", "bash", "sh", "shell"}
            continue
        if not in_block:
            continue
        if stripped.startswith("#") or stripped.startswith("$"):
            continue
        # Find all `heramind ...` substrings on the line (may be inside `for` loops etc.)
        for m in HERAMIND_RE.finditer(line):
            domain = m.group("domain")
            action = m.group("action")
            sub = m.group("sub")
            if action is None:
                continue  # `heramind foo` alone isn't a command shape we validate
            # Skip obvious IDs (UUIDs, hex, things-with-dashes that aren't subcommands)
            if (domain, action) in SUBDOMAINS:
                if sub and not _looks_like_id(sub):
                    found.add((domain, action, sub))
                else:
                    # Still record the (domain, action) pair even without valid sub
                    found.add((domain, action, None))
            else:
                # Only keep as a pair; the second word is usually an ID.
                found.add((domain, action, None))
    return found


def _looks_like_id(s: str) -> bool:
    """Heuristic: contains a dash + lowercase alphanumeric = probably an ID."""
    return "-" in s and not s.startswith("--")


def build_manifest() -> dict[str, list[list[str]]]:
    """Map {skill_id: [[domain, action, sub?], ...]}."""
    manifest: dict[str, list[list[str]]] = defaultdict(list)
    seen_per_skill: dict[str, set[tuple[str, str, str | None]]] = defaultdict(set)

    for path in sorted(SKILLS_DIR.glob(SKILL_GLOB)):
        skill_id = path.stem
        text = path.read_text(encoding="utf-8")
        for cmd in extract_commands(text):
            if cmd in seen_per_skill[skill_id]:
                continue
            seen_per_skill[skill_id].add(cmd)
            domain, action, sub = cmd
            entry = [domain, action]
            if sub is not None:
                entry.append(sub)
            manifest[skill_id].append(entry)

    return dict(manifest)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, help="Write JSON manifest to this path.")
    parser.add_argument("--pretty", action="store_true", help="Print manifest to stdout.")
    args = parser.parse_args()

    manifest = build_manifest()

    if args.out:
        args.out.write_text(json.dumps(manifest, indent=2, sort_keys=True))
        print(f"wrote {args.out} ({sum(len(v) for v in manifest.values())} commands across {len(manifest)} skills)")
    if args.pretty or not args.out:
        print(json.dumps(manifest, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
