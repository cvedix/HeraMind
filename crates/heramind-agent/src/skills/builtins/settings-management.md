---
id: settings-management
name: System Settings (Timezone & Data Retention)
description: Use when the user wants to change system settings — timezone, data retention, cleanup policies. Covers settings timezone/retention/cleanup even without saying 'settings' (e.g. '把时区改成上海', '数据保留30天'). Includes 设置/时区/数据保留/清理.
category: settings
origin: builtin
priority: 75
token_budget: 5000
triggers:
  keywords: [settings, 设置, timezone, 时区, set-timezone, 设置时区, timezones, 时区列表, retention, 数据保留, 保留期, set-retention, cleanup, 清理, data cleanup, 数据清理, heramind settings]
  tool_target:
    - tool: shell
      actions: [timezone, set-timezone, timezones, retention, set-retention, cleanup]
anti_triggers:
  keywords: [device create, 创建设备, rule create, 创建规则, dashboard, 仪表盘, system info, broker 地址]
---

# System Settings: Timezone & Data Retention

`heramind settings` manages the global timezone and telemetry data retention.

## CRITICAL Rules

1. **Timezone + retention live under `heramind settings`, NOT `heramind system`.** `heramind system set-timezone` / `heramind system retention` do NOT exist — use `heramind settings set-timezone` / `heramind settings retention`.
2. **Timezone values are IANA format** — e.g. `Asia/Shanghai`, `America/New_York`, `Europe/London`. List valid ones with `heramind settings timezones`.
3. **Retention is a number of days** controlling how long telemetry data is kept.

## Command Reference

| Command | Purpose |
|---|---|
| `heramind settings timezone` | Get the current global timezone |
| `heramind settings set-timezone <IANA>` | Set the timezone, e.g. `heramind settings set-timezone Asia/Shanghai` |
| `heramind settings timezones` | List available timezones |
| `heramind settings retention` | Get data retention configuration |
| `heramind settings set-retention <days>` | Update retention (in days), e.g. `heramind settings set-retention 30` |
| `heramind settings cleanup` | Trigger a manual data cleanup now |

### Examples

```bash
heramind settings set-timezone Asia/Shanghai
heramind settings timezones
heramind settings retention
heramind settings set-retention 30
heramind settings cleanup
```
