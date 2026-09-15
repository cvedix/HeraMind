// ---------------------------------------------------------------------------
// "List-only dead end" detection helpers
// ---------------------------------------------------------------------------

/// Action verbs (in Chinese and English) that indicate the user wants a mutation,
/// not just a query. When the original user message contains these but all executed
/// tool calls were read-only (list/get/latest/history), we detect a "list-only dead end"
/// and inject a forced continuation prompt.
const ACTION_VERBS: &[&str] = &[
    // Chinese
    "创建",
    "新建",
    "删除",
    "控制",
    "启用",
    "禁用",
    "启动",
    "停止",
    "更新",
    "修改",
    "开启",
    "关闭",
    "打开",
    "发送",
    "写入",
    "分享",
    "安装",
    "卸载",
    "移除",
    "批量启用",
    "批量删除",
    "批量创建",
    "全部启动",
    "添加",
    "替换",
    "绑定",
    "巡检",
    // English
    "create",
    "delete",
    "control",
    "enable",
    "disable",
    "start",
    "stop",
    "update",
    "turn on",
    "turn off",
    "switch",
    "send",
    "write",
    "share",
    "install",
    "uninstall",
    "remove",
    "add",
    "replace",
    "bind",
];

/// Check if the user message requests a mutation/action (not just a query).
pub(crate) fn user_message_requires_action(msg: &str) -> bool {
    let msg_lower = msg.to_lowercase();
    ACTION_VERBS.iter().any(|verb| msg_lower.contains(verb))
}

/// Check whether a message asks for live/recent HeraMind data that must be
/// retrieved with a read-only tool call before it can be answered.
///
/// Keep this separate from mutation detection: analytics questions should run
/// immediately, but must never be mistaken for permission to edit a dashboard.
pub(crate) fn user_message_requires_data_query(msg: &str) -> bool {
    let msg_lower = msg.to_lowercase();

    // Questions about how to use the product are documentation requests, not
    // requests to retrieve the current value.
    const HELP_PATTERNS: &[&str] = &[
        "how to",
        "how can",
        "làm sao",
        "làm thế nào",
        "cách hỏi",
        "hướng dẫn",
        "怎么",
        "如何",
    ];
    if HELP_PATTERNS
        .iter()
        .any(|pattern| msg_lower.contains(pattern))
    {
        return false;
    }

    const DATA_SUBJECTS: &[&str] = &[
        // Vietnamese
        "số xe",
        "phương tiện",
        "số lượng",
        "dữ liệu",
        "chỉ số",
        "metric",
        "telemetry",
        // English
        "vehicle",
        "count",
        "data",
        "metric",
        "telemetry",
        // Chinese
        "数据",
        "指标",
        "车辆",
        "数量",
        "遥测",
    ];
    const QUERY_TERMS: &[&str] = &[
        // Vietnamese
        "bao nhiêu",
        "đếm được",
        "so sánh",
        "tăng",
        "giảm",
        "hiện tại",
        "giờ qua",
        "trước đó",
        "lịch sử",
        "xu hướng",
        // English
        "how many",
        "compare",
        "increase",
        "decrease",
        "current",
        "last hour",
        "previous",
        "history",
        "trend",
        // Chinese
        "多少",
        "比较",
        "当前",
        "上一",
        "历史",
        "趋势",
    ];

    DATA_SUBJECTS
        .iter()
        .any(|subject| msg_lower.contains(subject))
        && QUERY_TERMS.iter().any(|term| msg_lower.contains(term))
}

/// Vehicle totals are derived telemetry, not dashboard metadata. A dashboard
/// list/get call can discover the binding, but it cannot return the measured
/// value, so these requests must continue through a telemetry history query.
pub(crate) fn user_message_requires_metric_history(msg: &str) -> bool {
    let msg_lower = msg.to_lowercase();
    const METRIC_SUBJECTS: &[&str] = &["số xe", "phương tiện", "vehicle", "车辆"];

    user_message_requires_data_query(msg)
        && METRIC_SUBJECTS
            .iter()
            .any(|subject| msg_lower.contains(subject))
}

/// Return whether the executed commands contain a query that can provide the
/// actual telemetry value. Discovery-only calls such as `dashboard list` and
/// `dashboard get/inspect` deliberately do not satisfy a vehicle-count request.
pub(crate) fn data_query_was_satisfied(
    user_message: &str,
    executed_commands: &[&str],
    has_any_tool_results: bool,
) -> bool {
    if !user_message_requires_data_query(user_message) {
        return true;
    }

    if !user_message_requires_metric_history(user_message) {
        return has_any_tool_results;
    }

    executed_commands.iter().any(|command| {
        let normalized = command.to_lowercase();
        normalized.contains("device history")
            || (normalized.contains("api get") && normalized.contains("telemetry"))
    })
}

/// Build a forced continuation for a read-only analytics request when the
/// model described what it would do but did not call any tool.
pub(crate) fn build_no_tool_data_query_prompt(user_message: &str) -> Option<String> {
    if !user_message_requires_data_query(user_message) {
        return None;
    }

    tracing::warn!(
        "No-tool analytics dead end detected. Injecting forced data-query continuation."
    );

    Some(format!(
        "⚠️ CRITICAL: The user's request below is a READ-ONLY DATA QUERY, but your previous \
response executed ZERO tools.\n\n\
Original request: {user_message}\n\n\
You MUST retrieve the real HeraMind data now. Do not describe a plan, do not ask for \
permission, and do not create or update a dashboard/widget.\n\
- Output a tool call NOW.\n\
- Use the `shell` tool with `heramind device history` for time-window metrics.\n\
- If the real device ID or metric is unknown, call `heramind device list` or \
`heramind device get <ID>` first; never guess them.\n\
- For a current-window versus previous-window comparison, execute both history queries \
in one JSON array. Use the same `--time-range` and `--aggregate`; add `--offset` only \
to the previous window.\n\
DO NOT output explanatory text before the tool call."
    ))
}

/// Build a forced continuation when discovery tools ran but the agent still
/// has not queried the telemetry value requested by the user.
pub(crate) fn build_incomplete_data_query_prompt(
    user_message: &str,
    executed_commands: &[&str],
    tool_results: &[(String, String)],
) -> Option<String> {
    if !user_message_requires_metric_history(user_message)
        || data_query_was_satisfied(user_message, executed_commands, true)
    {
        return None;
    }

    let executed = if executed_commands.is_empty() {
        "- none".to_string()
    } else {
        executed_commands
            .iter()
            .map(|command| format!("- {command}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    tracing::warn!(
        "Discovery-only analytics dead end detected. Requiring a telemetry history query."
    );

    let verified_next_command = resolve_metric_history_command(tool_results)
        .map(|command| {
            format!(
                "\nVERIFIED NEXT COMMAND (derived from successful dashboard/device results):\n\
`{command}`\n\
Execute this exact command now. Do not inspect or list again.\n"
            )
        })
        .unwrap_or_default();

    Some(format!(
        "⚠️ CRITICAL: The user asked for a REAL TELEMETRY VALUE, but the calls so far \
only discovered metadata and did not query that value.\n\n\
Original request: {user_message}\n\n\
Previously executed commands:\n{executed}\n\n\
You MUST continue with tool calls now. Do not give a summary yet.\n\
{verified_next_command}\
- If dashboard context identifies the relevant dashboard, use `heramind dashboard inspect <ID>` \
to discover the widget's real device/metric binding and configured time window.\n\
- Then call `heramind device history <DEVICE_ID> --metric <METRIC> --time-range <WINDOW> \
--aggregate <METHOD>` using the dashboard's configured aggregate to retrieve the count. \
Never guess IDs, metric names, the window, or the aggregation method.\n\
- A dashboard list/get/inspect result is configuration metadata, NOT the measured vehicle count.\n\
Output the next required tool call(s) now; do not output explanatory prose."
    ))
}

fn resolve_metric_history_command(tool_results: &[(String, String)]) -> Option<String> {
    let payloads = tool_results
        .iter()
        .filter_map(|(_, result)| {
            let outer: serde_json::Value = serde_json::from_str(result).ok()?;
            let cli = outer
                .get("stdout")
                .and_then(|stdout| stdout.as_str())
                .and_then(|stdout| serde_json::from_str::<serde_json::Value>(stdout).ok())
                .unwrap_or(outer);
            Some(cli.get("data").cloned().unwrap_or(cli))
        })
        .collect::<Vec<_>>();

    let mut metric = None;
    let mut time_range = None;
    let mut aggregate = None;
    let mut device_id = None;

    for payload in &payloads {
        let Some(components) = payload.get("components").and_then(|value| value.as_array()) else {
            continue;
        };
        for component in components {
            let title = component
                .get("title")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_lowercase();
            for source in data_source_objects(component.get("data_source")) {
                let candidate_metric = source
                    .get("metricId")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                let candidate_aggregate = source
                    .get("aggregateExt")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                let is_vehicle_total = (title.contains("phương tiện") || title.contains("vehicle"))
                    && (candidate_aggregate == "count"
                        || candidate_metric.ends_with("vehicle_seen"));
                if is_vehicle_total {
                    metric = valid_cli_token(candidate_metric).map(str::to_string);
                    time_range = source
                        .get("timeRange")
                        .and_then(|value| value.as_u64())
                        .map(|hours| format!("{hours}h"));
                    aggregate = match candidate_aggregate {
                        "avg" | "min" | "max" | "sum" | "count" | "last" => {
                            Some(candidate_aggregate.to_string())
                        }
                        _ => None,
                    };
                    if let Some(source_id) = source.get("sourceId").and_then(|value| value.as_str())
                    {
                        if !source_id.starts_with("transform:") {
                            device_id = valid_cli_token(source_id).map(str::to_string);
                        }
                    }
                    break;
                }
            }
        }
    }

    let metric = metric?;

    if device_id.is_none() {
        for payload in &payloads {
            let Some(types) = payload.get("types").and_then(|value| value.as_array()) else {
                continue;
            };
            for device_type in types {
                let supports_metric = device_type
                    .get("metric_fields")
                    .and_then(|value| value.as_array())
                    .is_some_and(|fields| {
                        fields.iter().any(|field| field.as_str() == Some(&metric))
                    });
                if !supports_metric {
                    continue;
                }
                device_id = device_type
                    .pointer("/devices/list/0/id")
                    .and_then(|value| value.as_str())
                    .and_then(valid_cli_token)
                    .map(str::to_string);
                if device_id.is_some() {
                    break;
                }
            }
        }
    }

    Some(format!(
        "heramind device history {} --metric {} --time-range {} --aggregate {}",
        device_id?,
        metric,
        time_range.unwrap_or_else(|| "24h".to_string()),
        aggregate.unwrap_or_else(|| "count".to_string())
    ))
}

fn data_source_objects(value: Option<&serde_json::Value>) -> Vec<&serde_json::Value> {
    match value {
        Some(serde_json::Value::Array(items)) => items.iter().collect(),
        Some(value @ serde_json::Value::Object(_)) => vec![value],
        _ => Vec::new(),
    }
}

fn valid_cli_token(value: &str) -> Option<&str> {
    (!value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':')))
    .then_some(value)
}

/// Return an honest localized failure instead of leaking a model-generated
/// plan when the bounded telemetry recovery attempts are exhausted.
pub(crate) fn build_data_query_failure_response(user_message: &str) -> String {
    let lower = user_message.to_lowercase();
    if [
        "số lượng",
        "phương tiện",
        "bao nhiêu",
        "đếm được",
        "dữ liệu",
    ]
    .iter()
    .any(|term| lower.contains(term))
    {
        "Không thể truy vấn giá trị telemetry thực sau nhiều lần thử. Tôi chưa có số liệu đáng tin cậy để trả lời; vui lòng thử lại hoặc kiểm tra kết nối model.".to_string()
    } else if ["数据", "指标", "车辆", "数量", "遥测"]
        .iter()
        .any(|term| lower.contains(term))
    {
        "多次尝试后仍无法查询真实遥测值。我目前没有可靠数据可供回答；请重试或检查模型连接。"
            .to_string()
    } else {
        "I could not retrieve the real telemetry value after multiple attempts. I do not have reliable data to answer with; please retry or check the model connection.".to_string()
    }
}

/// Check if ALL executed tool calls so far were read-only (list/get/query).
/// Takes the actual shell command strings (not tool names) for accurate detection.
/// Returns true if no mutation command was found in any tool call.
pub(crate) fn all_tools_were_read_only(
    executed_commands: &[&str],
    _all_results: &[(String, String)],
) -> bool {
    // Mutation command patterns — if ANY of these appear, it's NOT read-only
    const MUTATION_COMMANDS: &[&str] = &[
        " create",
        " delete",
        " update",
        " control",
        " enable",
        " disable",
        " write-metric",
        " send-message",
        " share",
        " install ",
        " uninstall ",
        " control ",
        " send ",
        " channel-create",
        " channel-update",
        " channel-delete",
        // Component-level mutations (dashboard widget edits) — without these,
        // an add-components/remove-components pair (a REAL mutation) reads as
        // "list/query only" and the forced continuation keeps firing after
        // the user's change is already applied.
        " add-component",
        " remove-component",
        " update-component",
        // Misc mutations
        " publish",
        " push",
        " register",
        " import",
    ];

    // If no commands were executed, we can't determine — assume not read-only
    if executed_commands.is_empty() {
        return false;
    }

    for cmd in executed_commands {
        let cmd_lower = cmd.to_lowercase();
        // Check if this command is a mutation
        let is_mutation = MUTATION_COMMANDS.iter().any(|m| cmd_lower.contains(m));
        if is_mutation {
            return false;
        }
    }

    // All commands were read-only (list/get/latest/history/etc.)
    true
}

/// Build the forced-continuation prompt for the "list-only dead end" case.
///
/// This is shared between the text-only path (`stream_core.rs`) and the
/// multimodal path (`stream_multimodal.rs`) so the prompt stays consistent.
///
/// Returns `Some(prompt)` if the user asked for an action but only read-only
/// tools were executed, otherwise `None` (caller proceeds with the normal
/// continuation prompt).
///
/// Arguments:
/// - `user_message`: original user message text
/// - `executed_commands`: actual shell command strings from prior tool calls
///   (used for the "Previously executed" summary AND the read-only check)
/// - `all_results`: `(tool_name, result_text)` accumulator across rounds
pub(crate) fn build_list_only_dead_end_prompt(
    user_message: &str,
    executed_commands: &[&str],
    all_results: &[(String, String)],
) -> Option<String> {
    if !user_message_requires_action(user_message) {
        return None;
    }
    if !all_tools_were_read_only(executed_commands, all_results) {
        return None;
    }

    let action_hint = extract_action_hint(user_message);

    // If we can't name the action, the verb match was almost certainly a
    // false positive — e.g. 「温度趋势的数据绑定好像不对?」 matches the
    // verb 绑定 as part of the NOUN 数据绑定 (data binding). Injecting an
    // unnamed "execute the action NOW" demand contradicts the model every
    // round and burns the whole iteration budget without producing text
    // (observed: 11 rounds / 17 tool calls / no final answer).
    if action_hint.is_empty() {
        tracing::debug!(
            "List-only dead end suspected, but no action hint could be extracted \
             (likely a noun false-positive on an action verb) — skipping forced continuation"
        );
        return None;
    }

    tracing::warn!(
        "List-only dead end detected! User wants action '{}' but only list/query tools were called. Injecting forced continuation.",
        action_hint
    );

    let executed_summary = if executed_commands.is_empty() {
        String::from("(no prior commands)")
    } else {
        executed_commands
            .iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut msg = format!(
        "⚠️ CRITICAL: The user asked you to perform an ACTION, but you ONLY ran list/query commands.\n\
        You MUST now execute the actual action command.\n\n\
        Previously executed (read-only):\n{}\n\n",
        executed_summary
    );

    if !action_hint.is_empty() {
        msg.push_str(&format!(
            "The user's original request requires: {}\n\
            You MUST output a tool call NOW to complete this action.\n\
            Use the IDs/data from the list results above to construct the command.\n\n",
            action_hint
        ));

        // Rule-specific: if creating a rule, verify device/metric discovery was done
        if action_hint.contains("rule") && action_hint.contains("create") {
            let has_device_list = executed_commands
                .iter()
                .any(|c| c.contains("device list") || c.contains("device get"));
            if !has_device_list {
                msg.push_str(
                    "⚠️ RULE CREATION REQUIRES METRIC DISCOVERY:\n\
                    You have NOT run `heramind device list` or `heramind device get <ID>` yet.\n\
                    You CANNOT create a rule without knowing the REAL device ID and metric field name.\n\
                    Run `heramind device list` FIRST to discover metric_fields, THEN construct the DSL.\n\
                    NEVER guess device IDs or metric names — they will silently fail.\n\n"
                );
            }
        }
    }

    msg.push_str(
        "DO NOT output text. DO NOT summarize the list results. DO NOT say 'I found the ...'.\n\
        OUTPUT A TOOL CALL JSON ARRAY NOW to execute the action.",
    );
    Some(msg)
}

/// Extract the domain and expected action from the user message for the forced prompt.
pub(crate) fn extract_action_hint(msg: &str) -> String {
    let msg_lower = msg.to_lowercase();

    // Domain detection
    let domain = if msg_lower.contains("规则") || msg_lower.contains("rule") {
        "rule"
    } else if msg_lower.contains("agent")
        || msg_lower.contains("代理")
        || msg_lower.contains("智能体")
    {
        "agent"
    } else if msg_lower.contains("设备")
        || msg_lower.contains("device")
        || msg_lower.contains("sensor")
    {
        "device"
    } else if msg_lower.contains("仪表盘")
        || msg_lower.contains("仪表板")
        || msg_lower.contains("dashboard")
        || msg_lower.contains("面板")
    {
        "dashboard"
    } else if msg_lower.contains("转换") || msg_lower.contains("transform") {
        "transform"
    } else if msg_lower.contains("组件")
        || msg_lower.contains("widget")
        || msg_lower.contains("小部件")
    {
        "widget"
    } else if msg_lower.contains("扩展")
        || msg_lower.contains("extension")
        || msg_lower.contains("插件")
    {
        "extension"
    } else if msg_lower.contains("消息")
        || msg_lower.contains("message")
        || msg_lower.contains("通知")
        || msg_lower.contains("通道")
        || msg_lower.contains("channel")
    {
        "message"
    } else {
        ""
    };

    // Action detection
    let action =
        if msg_lower.contains("创建") || msg_lower.contains("create") || msg_lower.contains("新建")
        {
            "create"
        } else if msg_lower.contains("删除")
            || msg_lower.contains("delete")
            || msg_lower.contains("移除")
        {
            "delete"
        } else if msg_lower.contains("控制")
            || msg_lower.contains("control")
            || msg_lower.contains("打开")
            || msg_lower.contains("关闭")
            || msg_lower.contains("开启")
        {
            "control"
        } else if msg_lower.contains("启用")
            || msg_lower.contains("enable")
            || msg_lower.contains("启动")
            || msg_lower.contains("start")
        {
            "enable/start"
        } else if msg_lower.contains("禁用")
            || msg_lower.contains("disable")
            || msg_lower.contains("停止")
            || msg_lower.contains("stop")
        {
            "disable/stop"
        } else if msg_lower.contains("更新")
            || msg_lower.contains("update")
            || msg_lower.contains("修改")
            || msg_lower.contains("替换")
        {
            "update"
        } else if msg_lower.contains("写入")
            || msg_lower.contains("write")
            || msg_lower.contains("发送")
            || msg_lower.contains("send")
        {
            "write/send"
        } else if msg_lower.contains("添加") || msg_lower.contains("add") {
            "add"
        } else if msg_lower.contains("安装") || msg_lower.contains("install") {
            "install"
        } else if msg_lower.contains("卸载") || msg_lower.contains("uninstall") {
            "uninstall"
        } else if msg_lower.contains("分享") || msg_lower.contains("share") {
            "share"
        } else {
            ""
        };

    if domain.is_empty() && action.is_empty() {
        String::new()
    } else if domain.is_empty() {
        format!("the {} action", action)
    } else if action.is_empty() {
        format!("heramind {}", domain)
    } else {
        format!("heramind {} {}", domain, action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message_requires_action_chinese() {
        assert!(user_message_requires_action("创建新设备"));
        assert!(user_message_requires_action("删除旧配置"));
        assert!(user_message_requires_action("启用监控"));
        assert!(user_message_requires_action("发送消息"));
        assert!(!user_message_requires_action("列出所有设备"));
        assert!(!user_message_requires_action("查看状态"));
    }

    #[test]
    fn test_user_message_requires_action_english() {
        assert!(user_message_requires_action("create device"));
        assert!(user_message_requires_action("delete old config"));
        assert!(user_message_requires_action("STOP the service"));
        assert!(!user_message_requires_action("list devices"));
        assert!(!user_message_requires_action("get status"));
    }

    #[test]
    fn test_user_message_requires_data_query_vietnamese() {
        assert!(user_message_requires_data_query(
            "So sánh số xe trong 1 giờ qua với 1 giờ trước đó"
        ));
        assert!(user_message_requires_data_query(
            "Hiện tại số lượng phương tiện là bao nhiêu?"
        ));
        assert!(user_message_requires_data_query(
            "Số lượng phương tiện đếm được"
        ));
        assert!(!user_message_requires_data_query(
            "Làm sao người dùng có thể hỏi AI về số lượng phương tiện?"
        ));
        assert!(!user_message_requires_data_query(
            "Tạo biểu đồ số lượng phương tiện"
        ));
    }

    #[test]
    fn test_build_no_tool_data_query_prompt() {
        let prompt =
            build_no_tool_data_query_prompt("So sánh số xe trong 1 giờ qua với 1 giờ trước đó")
                .expect("analytics request should force a tool call");
        assert!(prompt.contains("ZERO tools"));
        assert!(prompt.contains("heramind device history"));
        assert!(prompt.contains("--offset"));

        assert!(build_no_tool_data_query_prompt(
            "Làm sao người dùng có thể hỏi AI về số lượng phương tiện?"
        )
        .is_none());
    }

    #[test]
    fn test_vehicle_count_requires_real_metric_history() {
        let request = "Số lượng phương tiện đếm được";
        assert!(user_message_requires_metric_history(request));
        assert!(!data_query_was_satisfied(
            request,
            &["heramind dashboard list"],
            true
        ));
        assert!(!data_query_was_satisfied(
            request,
            &[
                "heramind dashboard list",
                "heramind dashboard get dashboard-1"
            ],
            true
        ));
        assert!(data_query_was_satisfied(
            request,
            &[
                "heramind dashboard get dashboard-1",
                "heramind device history camera-1 --metric vehicle_seen --time-range 24h --aggregate sum"
            ],
            true
        ));

        let prompt = build_incomplete_data_query_prompt(request, &["heramind dashboard list"], &[])
            .expect("dashboard metadata alone must force a telemetry query");
        assert!(prompt.contains("dashboard inspect"));
        assert!(prompt.contains("device history"));
        assert!(prompt.contains("NOT the measured vehicle count"));
        assert!(build_data_query_failure_response(request).starts_with("Không thể truy vấn"));
    }

    #[test]
    fn test_all_tools_were_read_only() {
        assert!(all_tools_were_read_only(
            &["heramind device list", "heramind rule list"],
            &[]
        ));
        assert!(!all_tools_were_read_only(
            &["heramind device create sensor1"],
            &[]
        ));
        assert!(!all_tools_were_read_only(
            &["heramind device list", "heramind device delete sensor1"],
            &[]
        ));
        assert!(!all_tools_were_read_only(&[], &[])); // empty = not read-only
    }

    #[test]
    fn test_extract_action_hint() {
        assert_eq!(extract_action_hint("创建新规则"), "heramind rule create");
        assert_eq!(extract_action_hint("删除设备"), "heramind device delete");
        assert_eq!(
            extract_action_hint("创建仪表盘"),
            "heramind dashboard create"
        );
        assert_eq!(extract_action_hint("查看状态"), "");
    }

    #[test]
    fn test_noun_verb_collision_skips_forced_continuation() {
        // Real incident (2026-08-22): 「温度趋势的数据绑定好像不对?」— the
        // noun 数据绑定 contains the ACTION_VERB 绑定, so requires_action
        // matched, but extract_action_hint found no action. The injected
        // "execute the action NOW" prompt then contradicted the model every
        // round → 11 rounds / 17 tool calls / no final text.
        let msg = "温度趋势的数据绑定好像不对?";
        assert!(
            user_message_requires_action(msg),
            "verb collision still matches"
        );
        assert_eq!(extract_action_hint(msg), "");

        // Read-only investigation commands → dead-end condition holds, but
        // the empty hint must suppress the injection.
        let cmds = [
            "heramind widget get sparkline",
            "heramind device get demo-001",
        ];
        assert!(all_tools_were_read_only(&cmds, &[]));
        assert!(build_list_only_dead_end_prompt(msg, &cmds, &[]).is_none());
    }

    #[test]
    fn test_named_action_still_injects() {
        // Guard must not over-suppress: a genuine action request with a
        // nameable action still injects.
        let msg = "创建一个温湿度仪表盘";
        let cmds = ["heramind device list", "heramind widget list"];
        assert!(build_list_only_dead_end_prompt(msg, &cmds, &[]).is_some());
    }

    #[test]
    fn test_component_mutations_are_not_read_only() {
        // Incident addendum: an add-components + remove-components pair IS a
        // mutation — the detector used to keep firing after the change was
        // already applied, burning the turn to the 5-minute wall clock.
        assert!(!all_tools_were_read_only(
            &["heramind dashboard add-components d1 --components '[]'"],
            &[]
        ));
        assert!(!all_tools_were_read_only(
            &["heramind dashboard remove-components d1 --ids '[\"c3\"]'"],
            &[]
        ));
        assert!(!all_tools_were_read_only(
            &["heramind dashboard update-component d1 --id c3 --set '{}'"],
            &[]
        ));
        // Query commands remain read-only.
        assert!(all_tools_were_read_only(
            &["heramind dashboard get d1", "heramind widget list"],
            &[]
        ));
    }
}
