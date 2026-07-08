//! Tool call parser for extracting tool calls from LLM responses.
//!
//! Priority: JSON > XML (fallback)
//! JSON format preserves tool IDs from Ollama/OpenAI API.

use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;
use uuid::Uuid;

use super::types::ToolCall;
use crate::error::Result;

/// Pre-compiled regex for removing code-block-wrapped tool call arrays from responses.
fn code_block_array_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"```(?:json)?\s*\n?\s*(\[\s*\{[\s\S]*?"name"[\s\S]*?\}\s*\])\s*\n?\s*```"#)
            .expect("code block tool call regex is a compile-time constant")
    })
}

/// Pre-compiled regex for removing code-block-wrapped single tool call objects from responses.
fn code_block_obj_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"```(?:json)?\s*\n?\s*(\{\s*"name"[\s\S]*?"arguments"[\s\S]*?\})\s*\n?\s*```"#)
            .expect("code block single tool call regex is a compile-time constant")
    })
}

/// Parse tool calls from LLM response text.
///
/// **Supported formats** (in priority order):
/// 1. JSON array: `[{"id": "call_123", "name": "tool1", "arguments": {...}}]`
/// 2. JSON object: `{"id": "call_123", "name": "tool_name", "arguments": {...}}`
/// 3. XML (fallback): `<tool_calls><invoke name="tool_name">...</invoke></tool_calls>`
///
/// Returns the remaining text along with any parsed tool calls.
pub fn parse_tool_calls(text: &str) -> Result<(String, Vec<ToolCall>)> {
    // === PRIORITY 1: JSON array format ===
    // Native format from Ollama/OpenAI, preserves tool IDs
    if let Some(result) = try_parse_json_array(text) {
        return result;
    }

    // === PRIORITY 2: JSON object format ===
    if let Some(result) = try_parse_json_object(text) {
        return result;
    }

    // === PRIORITY 3: XML format (fallback for models without native tool support) ===
    if let Some(result) = try_parse_xml(text) {
        return result;
    }

    Ok((text.to_string(), Vec::new()))
}

/// Try to parse JSON array format tool calls.
/// Returns None if not found, Some(result) if found (even if empty).
fn try_parse_json_array(text: &str) -> Option<Result<(String, Vec<ToolCall>)>> {
    let start = text.find('[')?;

    // Find matching closing bracket
    let mut bracket_count = 0;
    let mut end = start;
    for (i, c) in text[start..].char_indices() {
        match c {
            '[' => bracket_count += 1,
            ']' => {
                bracket_count -= 1;
                if bracket_count == 0 {
                    end = start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end <= start {
        return None;
    }

    let json_str = &text[start..end];

    // Check if it looks like tool calls
    if !json_str.contains("\"name\"")
        && !json_str.contains("\"tool\"")
        && !json_str.contains("\"function\"")
    {
        return None;
    }

    let array = serde_json::from_str::<Vec<Value>>(json_str).ok()?;

    let mut tool_calls = Vec::new();
    for value in array {
        if let Some(tool_call) = extract_tool_call_from_json(&value) {
            tool_calls.push(tool_call);
        }
    }

    if tool_calls.is_empty() {
        return None;
    }

    let content = text[..start].trim().to_string();
    Some(Ok((content, tool_calls)))
}

/// Try to parse JSON object format tool call.
fn try_parse_json_object(text: &str) -> Option<Result<(String, Vec<ToolCall>)>> {
    let start = text.find('{')?;

    // Find matching closing brace
    let mut brace_count = 0;
    let mut end = start;
    for (i, c) in text[start..].char_indices() {
        match c {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    end = start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end <= start {
        return None;
    }

    let json_str = &text[start..end];
    let value = serde_json::from_str::<Value>(json_str).ok()?;

    if let Some(tool_call) = extract_tool_call_from_json(&value) {
        let content = text[..start].trim().to_string();
        return Some(Ok((content, vec![tool_call])));
    }

    None
}

/// Extract a ToolCall from a JSON value.
/// Preserves the `id` field from Ollama/OpenAI API.
fn extract_tool_call_from_json(value: &Value) -> Option<ToolCall> {
    // Get tool name from various possible fields
    let name = value
        .get("name")
        .or_else(|| value.get("tool"))
        .or_else(|| value.get("function"))
        .and_then(|v| v.as_str())?
        .to_string();

    // Preserve the ID from API, or generate a new one
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Get arguments
    let arguments = value
        .get("arguments")
        .or_else(|| value.get("params"))
        .or_else(|| value.get("parameters"))
        .cloned()
        .unwrap_or_else(|| {
            // If no explicit arguments, collect remaining fields
            let mut args = serde_json::Map::new();
            if let Some(obj) = value.as_object() {
                for (k, v) in obj {
                    if !matches!(
                        k.as_str(),
                        "name" | "tool" | "function" | "arguments" | "params" | "parameters" | "id"
                    ) {
                        args.insert(k.clone(), v.clone());
                    }
                }
            }
            Value::Object(args)
        });

    Some(ToolCall {
        name,
        id,
        arguments,
        result: None,
        round: None,
    })
}

/// Try to parse XML format tool calls (fallback for models without native tool support).
fn try_parse_xml(text: &str) -> Option<Result<(String, Vec<ToolCall>)>> {
    let start = text.find("<tool_calls>")?;
    let end = text.find("</tool_calls>")?;

    let xml_section = &text[start..end + 13];
    let content = format!("{}{}", &text[..start], &text[end + 13..]);

    let mut tool_calls = Vec::new();
    let mut remaining = xml_section;

    while let Some(invoke_start) = remaining.find("<invoke") {
        let invoke_end = remaining.find("</invoke>")?;
        let invoke_section = &remaining[invoke_start..invoke_end + 8];

        // Extract tool name
        if let Some(tool_call) = parse_invoke_element(invoke_section) {
            tool_calls.push(tool_call);
        }

        remaining = &remaining[invoke_end + 8..];
    }

    if tool_calls.is_empty() {
        return None;
    }

    Some(Ok((content.trim().to_string(), tool_calls)))
}

/// Parse a single <invoke> element from XML.
fn parse_invoke_element(invoke_section: &str) -> Option<ToolCall> {
    let name_start = invoke_section.find("name=\"")?;
    let name_section = &invoke_section[name_start + 6..];
    let name_end = name_section.find('"')?;
    let tool_name = &name_section[..name_end];

    // Extract parameters
    let mut arguments = serde_json::Map::new();
    let mut search_start = 0;

    while search_start < invoke_section.len() {
        if let Some(param_start) = invoke_section[search_start..].find("<parameter") {
            let absolute_param_start = search_start + param_start;

            // Find end of parameter tag
            let tag_end = invoke_section[absolute_param_start..].find('>')?;
            let absolute_tag_end = absolute_param_start + tag_end;
            let tag_section = &invoke_section[absolute_param_start..=absolute_tag_end];
            let is_self_closing = tag_section.trim_end().ends_with("/>");

            // Extract parameter name
            let param_name = if let Some(n_start) = tag_section.find("name=\"") {
                let n_section = &tag_section[n_start + 6..];
                if let Some(n_end) = n_section.find('"') {
                    n_section[..n_end].to_string()
                } else {
                    search_start = absolute_param_start + "<parameter".len();
                    continue;
                }
            } else {
                search_start = absolute_param_start + "<parameter".len();
                continue;
            };

            // Extract parameter value
            if let Some(v_start) = tag_section.find("value=\"") {
                let v_section = &tag_section[v_start + 7..];
                if let Some(v_end) = v_section.find('"') {
                    arguments.insert(param_name, Value::String(v_section[..v_end].to_string()));
                }
                search_start = absolute_tag_end + 1;
            } else if !is_self_closing {
                // Content format: <parameter name="key">value</parameter>
                let content_start = absolute_tag_end + 1;
                if let Some(close_end) = invoke_section[content_start..].find("</parameter>") {
                    let value = invoke_section[content_start..content_start + close_end]
                        .trim()
                        .to_string();
                    arguments.insert(param_name, Value::String(value));
                    search_start = content_start + close_end + "</parameter>".len();
                } else {
                    search_start = absolute_param_start + "<parameter".len();
                }
            } else {
                search_start = absolute_tag_end + 1;
            }
        } else {
            break;
        }
    }

    Some(ToolCall {
        name: tool_name.to_string(),
        id: Uuid::new_v4().to_string(), // XML format doesn't have IDs
        arguments: Value::Object(arguments),
        result: None,
        round: None,
    })
}

/// Remove tool call markers from response for memory storage.
pub fn remove_tool_calls_from_response(response: &str) -> String {
    let mut result = response.to_string();

    // Remove ```json ... ``` code blocks that contain tool call JSON
    result = code_block_array_re().replace_all(&result, "").to_string();

    // Also remove ```json ... ``` with single object tool calls
    result = code_block_obj_re().replace_all(&result, "").to_string();

    // Remove JSON array format
    while let Some(start) = result.find('[') {
        let mut bracket_count = 0;
        let mut end = start;

        for (i, c) in result[start..].char_indices() {
            match c {
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end > start {
            let json_str = &result[start..end];
            if let Ok(array) = serde_json::from_str::<Vec<Value>>(json_str) {
                if array
                    .iter()
                    .any(|v| v.get("name").is_some() || v.get("tool").is_some())
                {
                    result.replace_range(start..end, "");
                    continue;
                }
            }
        }
        break;
    }

    // Remove JSON object format
    while let Some(start) = result.find('{') {
        let mut brace_count = 0;
        let mut end = start;

        for (i, c) in result[start..].char_indices() {
            match c {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end > start {
            let json_str = &result[start..end];
            if let Ok(value) = serde_json::from_str::<Value>(json_str) {
                if value.get("name").is_some() || value.get("tool").is_some() {
                    result.replace_range(start..end, "");
                    continue;
                }
            }
        }
        break;
    }

    // Remove XML format
    while let Some(start) = result.find("<tool_calls>") {
        if let Some(end) = result.find("</tool_calls>") {
            result.replace_range(start..end + 13, "");
            continue;
        }
        break;
    }

    result.trim().to_string()
}

/// Detect "degenerate" model output: a response whose every non-blank line is a
/// markdown code-fence marker (```, optionally followed by a language tag like
/// ```json) with no actual prose content anywhere.
///
/// DeepSeek-class models occasionally emit just ` ``` ` (or an empty ` ```\n``` `
/// pair) as their entire final answer when they intend to format a summary but
/// stop immediately after the fence opener. Left alone this produces a useless
/// empty reply that tanks `response_quality` and `language_adherence` scores.
///
/// Returns `true` when the response carries no surfaceable content, so callers
/// can trigger their existing empty-response recovery (e.g. retry without
/// thinking). Safe for normal responses: any non-fence line (prose, JSON,
/// code body) makes this return `false`.
pub fn is_degenerate_fence_only_output(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return true;
    }
    let mut saw_fence = false;
    for line in trimmed.lines() {
        let lt = line.trim();
        if lt.is_empty() {
            continue;
        }
        if let Some(after) = lt.strip_prefix("```") {
            // A fence marker is "```" optionally followed by a language tag
            // consisting solely of [A-Za-z0-9-+_].
            if after.is_empty()
                || after
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '+' || c == '_')
            {
                saw_fence = true;
                continue;
            }
        }
        // Any other non-blank line means the response has real content.
        return false;
    }
    saw_fence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degenerate_fence_only_single_marker() {
        assert!(is_degenerate_fence_only_output("```"));
        assert!(is_degenerate_fence_only_output("  ```\n"));
        assert!(is_degenerate_fence_only_output("\n```  "));
    }

    #[test]
    fn test_degenerate_fence_only_empty_pair() {
        assert!(is_degenerate_fence_only_output("```\n```"));
        assert!(is_degenerate_fence_only_output("```\n\n```"));
        assert!(is_degenerate_fence_only_output("```json\n```"));
    }

    #[test]
    fn test_degenerate_fence_only_with_language_tags() {
        assert!(is_degenerate_fence_only_output("```python\n```bash"));
        assert!(is_degenerate_fence_only_output("```json"));
    }

    #[test]
    fn test_degenerate_not_triggered_for_real_content() {
        // Prose anywhere → not degenerate.
        assert!(!is_degenerate_fence_only_output(
            "Done. The file is 45 bytes."
        ));
        // Code fence WITH body content → not degenerate.
        assert!(!is_degenerate_fence_only_output("```json\n{\"a\": 1}\n```"));
        assert!(!is_degenerate_fence_only_output(
            "Here is the result:\n```json\n{\"a\": 1}\n```"
        ));
        // Prose wrapping a fence → not degenerate.
        assert!(!is_degenerate_fence_only_output(
            "Summary:\n```\nsome output\n```\nThat's it."
        ));
        // Empty input is the only overlap — treated as degenerate so recovery runs.
        assert!(is_degenerate_fence_only_output(""));
        assert!(is_degenerate_fence_only_output("   \n  "));
    }

    #[test]
    fn test_parse_json_array_with_id() {
        let text =
            r#"[{"id": "call_abc123", "name": "list_devices", "arguments": {"type": "sensor"}}]"#;
        let (content, calls) = parse_tool_calls(text).unwrap();

        assert!(content.is_empty());
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "list_devices");
        assert_eq!(calls[0].id, "call_abc123"); // ID preserved!
        assert_eq!(calls[0].arguments["type"], "sensor");
    }

    #[test]
    fn test_parse_json_array_without_id() {
        let text = r#"[{"name": "list_devices", "arguments": {}}]"#;
        let (_content, calls) = parse_tool_calls(text).unwrap();

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "list_devices");
        // ID should be generated (UUID format)
        assert!(!calls[0].id.is_empty());
    }

    #[test]
    fn test_parse_json_object_with_id() {
        let text =
            r#"{"id": "call_xyz", "name": "query_data", "arguments": {"device": "sensor1"}}"#;
        let (_content, calls) = parse_tool_calls(text).unwrap();

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_xyz"); // ID preserved!
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let text =
            r#"[{"id": "call_1", "name": "list_devices"}, {"id": "call_2", "name": "list_rules"}]"#;
        let (_, calls) = parse_tool_calls(text).unwrap();

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[1].id, "call_2");
    }

    #[test]
    fn test_parse_xml_fallback() {
        let text = r#"<tool_calls><invoke name="device.query"><parameter name="device_id">sensor1</parameter></invoke></tool_calls>"#;
        let (_content, calls) = parse_tool_calls(text).unwrap();

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "device.query");
        assert_eq!(calls[0].arguments["device_id"], "sensor1");
        // XML format generates UUID
        assert!(!calls[0].id.is_empty());
    }

    #[test]
    fn test_json_priority_over_xml() {
        // When both formats exist, JSON should be parsed first
        let text = r#"[{"id": "call_json", "name": "list_devices"}]<tool_calls><invoke name="list_rules"></invoke></tool_calls>"#;
        let (_, calls) = parse_tool_calls(text).unwrap();

        // Should parse JSON, not XML
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_json");
        assert_eq!(calls[0].name, "list_devices");
    }

    #[test]
    fn test_parse_tool_calls_no_tools() {
        let text = "Hello, how can I help you today?";
        let (content, calls) = parse_tool_calls(text).unwrap();

        assert_eq!(content, text);
        assert_eq!(calls.len(), 0);
    }

    #[test]
    fn test_parse_with_content() {
        let text = r#"Let me check. [{"id": "call_1", "name": "list_devices"}]"#;
        let (content, calls) = parse_tool_calls(text).unwrap();

        assert_eq!(content, "Let me check.");
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn test_remove_tool_calls() {
        let response = r#"Checking... [{"id": "call_1", "name": "test"}] done"#;
        let cleaned = remove_tool_calls_from_response(response);

        assert!(cleaned.contains("Checking..."));
        assert!(cleaned.contains("done"));
        assert!(!cleaned.contains("call_1"));
    }
}
