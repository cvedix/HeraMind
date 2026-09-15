---
id: extension-management
name: Extension Management (Install/Market/Status)
description: Use when the user wants to INSTALL or MANAGE existing extensions — installing/uninstalling from market, listing, status, logs, config, reload. Distinct from DEVELOPMENT (that's extension-development). Covers extension install/market/status even without saying 'extension' (e.g. '装个天气扩展'). Includes 扩展安装/卸载/市场/状态.
category: extension
origin: builtin
priority: 90
token_budget: 4500
triggers:
  keywords: [extension install, extension list, extension market, marketplace, market-install, market-list, extension status, extension logs, extension get, extension validate, extension config, extension reload, extension uninstall, 安装扩展, 卸载扩展, 扩展列表, 扩展市场, 扩展状态, 扩展日志, 扩展配置, heramind extension]
  tool_target:
    - tool: shell
      actions: [install, uninstall, list, get, status, logs, validate, config, reload, market-install, market-list]
anti_triggers:
  keywords: [create extension, build extension, extension sdk, heramind_export, FFI, Rust, 扩展开发, 开发扩展, scaffold, manifest.json]
---

# Extension Management (Install / Market / Status)

Manage installed extensions and the marketplace. (For *developing/building* an extension from source, see the `extension-development` skill.)

## CRITICAL Rules

1. **Install from a local `.nep` file** → `heramind extension install <path.nep>`. **Install from the marketplace** → `heramind extension market-install <id>` — these are different commands.
2. **Always `validate` a local `.nep` before `install`** — `heramind extension validate <path.nep>`.
3. **RUN the command yourself and report the output** — don't just tell the user to run it.
4. After install/reload, check health with `heramind extension status <id>`; debug with `heramind extension logs <id>`.

## Command Cheat-Sheet

| Command | Purpose |
|---|---|
| `heramind extension list` | List installed extensions |
| `heramind extension get <id>` | Show one extension's info (alias: `info`) |
| `heramind extension status <id>` | Extension health/status |
| `heramind extension logs <id>` | Extension logs (debug crashes/errors) |
| `heramind extension config <id>` | Get/set extension configuration |
| `heramind extension reload <id>` | Reload an extension after changes |
| `heramind extension validate <path.nep>` | Validate a local `.nep` package before install |
| `heramind extension install <path.nep>` | Install a local `.nep` package |
| `heramind extension uninstall <id>` | Uninstall an extension |
| `heramind extension market-list` | List extensions available in the marketplace |
| `heramind extension market-install <id>` | Install an extension from the marketplace |
| `heramind extension create` | Scaffold a NEW extension (dev — see `extension-development` skill) |
| `heramind extension build` | Build extension from source (dev — see `extension-development` skill) |

### Examples

```bash
heramind extension list
heramind extension market-list
heramind extension market-install weather
heramind extension validate ./my-ext-1.0.0.nep
heramind extension install ./my-ext-1.0.0.nep
heramind extension status weather
heramind extension logs weather
heramind extension config weather
```
