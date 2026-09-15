---
id: widget-management
name: Widget Management (Install/Market/List)
description: Use when the user wants to INSTALL or MANAGE existing dashboard widgets — installing from market, listing, getting bundle, uninstalling. Distinct from DEVELOPMENT (that's widget-development). Covers widget install/market/bundle even without saying 'widget' (e.g. '装个仪表盘组件'). Includes 组件安装/卸载/市场.
category: widget
origin: builtin
priority: 90
token_budget: 4500
triggers:
  keywords: [widget list, widget get, widget bundle, widget install, widget uninstall, widget market, widget market-install, widget market-list, dashboard widget, 安装组件, 卸载组件, 组件列表, 组件市场, 市场组件, 获取组件, heramind widget]
  tool_target:
    - tool: shell
      actions: [list, get, bundle, install, uninstall, market-install, market-list]
anti_triggers:
  keywords: [create widget, scaffold widget, IIFE, bundle.js, React, 开发组件, 开发 widget, manifest.json, dashboard component develop]
---

# Widget Management (Install / Market / List)

Manage dashboard widgets and the widget marketplace. (For *developing* a widget from scratch, see the `widget-development` skill.)

## CRITICAL Rules

1. **Install from a local scaffold/zip** → `heramind widget install <path>`. **Install from the marketplace** → `heramind widget market-install <id>` — different commands.
2. **`heramind widget bundle <id>`** returns the widget's JS/CSS bundle (for inspection/embedding) — NOT `file_write`/`file_read`/`web_fetch`.
3. **`heramind widget market-list`** lists marketplace widgets; **`heramind widget list`** lists already-installed ones.
4. RUN the command yourself and report the output — don't narrate.

## Command Cheat-Sheet

| Command | Purpose |
|---|---|
| `heramind widget list` | List installed widgets |
| `heramind widget get <id>` | Show widget details |
| `heramind widget bundle <id>` | Get a widget's bundle (JS/CSS) |
| `heramind widget install <path>` | Install from a scaffolded dir or `.zip` |
| `heramind widget uninstall <id>` | Uninstall a widget |
| `heramind widget market-list` | List widgets available in the marketplace |
| `heramind widget market-install <id>` | Install a widget from the marketplace |
| `heramind widget create` | Scaffold a NEW widget (dev — see `widget-development` skill) |

### Examples

```bash
heramind widget list
heramind widget market-list
heramind widget market-install gauge-chart
heramind widget bundle gauge-chart
heramind widget get gauge-chart
heramind widget install ./my-widget/
heramind widget uninstall gauge-chart
```
