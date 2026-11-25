---
id: extension-management
summary: Install, configure, diagnose, and operate installed extensions (status/logs/commands).
not_for: [writing extension code (extension-development)]
name: Extension Management (Install/Config/Diagnose)
aliases: [extension, extensions]
description: Use when the user wants to INSTALL, CONFIGURE, or DIAGNOSE extensions — marketplace/local install, config, status, logs, reload, list/commands/invoke. NOT for writing extension source code (that's extension-development). Covers install/market/status/logs requests even without the word 'extension' (e.g. 'install the weather extension', '装个天气扩展', '扩展怎么了'). FIRST check whether it's already installed — installed extensions may simply need configuration or diagnosis.
category: extension
origin: builtin
priority: 90
token_budget: 4500
triggers:
  keywords: [extension install, install extension, add extension, extension market, marketplace, market install, market-list, install from market, install weather extension, weather extension, market-install, extension status, extension logs, extension get, extension validate, extension config, extension reload, extension uninstall, already installed, 安装扩展, 卸载扩展, 扩展列表, 扩展市场, 扩展状态, 扩展日志, 扩展配置, 已经安装, heramind extension]
  tool_target:
    - tool: shell
      actions: [install, uninstall, list, get, status, logs, validate, config, reload, market-install, market-list]
anti_triggers:
  keywords: [create extension, build extension, extension sdk, heramind_export, FFI, Rust, 扩展开发, 开发扩展, scaffold, manifest.json]
---

# extension-management

Cover the full lifecycle: install (market or local `.nep`), configure, diagnose (status/logs), and operate (list/commands/invoke) — not marketplace-only.
Inspect installed extensions and available packages. For install/update, use the intended package and verify compatibility, enablement and exposed commands/metrics.
Selection for invoke: "what extensions / what can you do" → `heramind extension list` (do not `--search` for "extension capabilities"); concrete task → `heramind extension commands --search "<task>"` then paste hit `shell_example` `{command, params}`; known id+command → reuse invoke; full schema / `requires_schema_lookup` → `heramind extension commands <id>`. Do not invent sanitized chat-tool names as CLI subcommands.
Do not confuse extension installation with skill loading or widget installation. Preserve configuration during upgrades and report restart/enable requirements.
Verify extension status and actual command/metric results. Load installation, configuration, or diagnosis reference sections as needed.

## Commands
| Command | Purpose |
|---|---|
| `heramind extension list` | List installed extensions |
| `heramind extension get <id>` | Show one extension's info (alias: `info`) |
| `heramind extension commands --search "<task>"` | Rank AI-visible commands for a task (≤3; purpose + required params) |
| `heramind extension commands <id>` | Machine-readable command schemas + invoke examples |
| shell `{command:"heramind extension invoke <id> <command>", params:{…}}` | Run a command (prefer structured `params`; quoted `--params '{…}'` is CLI fallback) |
| `heramind extension status <id>` | Extension health/status |
| `heramind extension logs <id>` | Extension logs (debug crashes/errors) |
| `heramind extension config <id>` | Get/set extension configuration |
| `heramind extension reload <id>` | Reload an extension after changes |
| `heramind extension validate <path.nep>` | Validate a local `.nep` package before install |
| `heramind extension install <path.nep>` | Install a local `.nep` package |
| `heramind extension uninstall <id>` | Uninstall an extension |
| `heramind extension market-list <关键词>` | Search the marketplace (compact: id/name/version). ALWAYS give a keyword — the unfiltered dump is huge. `-v` for full metadata |
| `heramind extension market-install <id>` | Install an extension from the marketplace |
| `heramind extension create` | Scaffold a NEW extension (dev — see `extension-development` skill) |
| `heramind extension build` | Build extension from source (dev — see `extension-development` skill) |

## Detailed reference
Use skill(action="load", id="extension-management", section="<exact returned title>") only for the needed detail. Section titles are returned with this overview.

<!-- SKILL_REFERENCE -->


# Extension Management (Install / Config / Diagnose)

Install, configure, diagnose (status/logs), and invoke installed extensions — marketplace is one install path, not the whole skill. (For *developing/building* an extension from source, see the `extension-development` skill.)

## CRITICAL Rules

1. **CHECK FIRST — it may already be installed**: run `heramind extension list` and look for the requested id. If present, DO NOT install anything — tell the user it's installed (show version), and offer `heramind extension config <id>` if it needs a key or setting. Installing over an existing id is not an idempotent no-op.
2. **Install from a local `.nep` file** → `heramind extension install <path.nep>`. **Install from the marketplace** → `heramind extension market-install <id>` — these are different commands.
3. **Always `validate` a local `.nep` before `install`** — `heramind extension validate <path.nep>`.
4. **RUN the command yourself and report the output** — don't just tell the user to run it.
5. After install/reload, check health with `heramind extension status <id>`; debug with `heramind extension logs <id>`.
6. **Prefer structured shell `params` for invoke** — `{"command":"heramind extension invoke <id> <cmd>", "params":{…}}`. Avoid raw JSON inside the command string.
7. **Marketplace install is ALWAYS exactly three commands, run each ONCE:**
   1. `heramind extension market-list <用户要的东西>` — pick the exact `id`
   2. `heramind extension market-install <id>`
   3. `heramind extension list` — confirm it appears; report success to the user
   Do NOT re-run `market-list` (the output is large), do NOT load this skill twice, and do NOT summarize the market listing — the user asked to INSTALL, so install.

## Command Cheat-Sheet

| Command | Purpose |
|---|---|
| `heramind extension list` | List installed extensions |
| `heramind extension get <id>` | Show one extension's info (alias: `info`) |
| `heramind extension commands --search "<task>"` | Rank AI-visible commands for a task |
| `heramind extension commands <id>` | Command schemas + invoke examples |
| shell `{command:"heramind extension invoke <id> <command>", params:{…}}` | Run a command (prefer structured `params`) |
| `heramind extension status <id>` | Extension health/status |
| `heramind extension logs <id>` | Extension logs (debug crashes/errors) |
| `heramind extension config <id>` | Get/set extension configuration |
| `heramind extension reload <id>` | Reload an extension after changes |
| `heramind extension validate <path.nep>` | Validate a local `.nep` package before install |
| `heramind extension install <path.nep>` | Install a local `.nep` package |
| `heramind extension uninstall <id>` | Uninstall an extension |
| `heramind extension market-list <关键词>` | Search the marketplace (compact: id/name/version). ALWAYS give a keyword — the unfiltered dump is huge. `-v` for full metadata |
| `heramind extension market-install <id>` | Install an extension from the marketplace |
| `heramind extension create` | Scaffold a NEW extension (dev — see `extension-development` skill) |
| `heramind extension build` | Build extension from source (dev — see `extension-development` skill) |

### Examples

```bash
heramind extension list
heramind extension commands --search "weather in Xiamen"
heramind extension commands weather-forecast-v2
# Prefer structured shell params: {"command":"heramind extension invoke weather-forecast-v2 get_weather","params":{"city":"Xiamen"}}
heramind extension invoke weather-forecast-v2 get_weather --params '{"city":"Xiamen"}'
heramind extension market-list weather     # compact, filtered
heramind extension market-install weather-forecast   # install the EXACT id shown
heramind extension validate ./my-ext-1.0.0.nep
heramind extension install ./my-ext-1.0.0.nep
heramind extension status weather
heramind extension logs weather
heramind extension config weather
```
