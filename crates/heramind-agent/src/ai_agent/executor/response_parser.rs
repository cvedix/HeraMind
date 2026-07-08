pub(crate) fn extract_command_from_description(description: &str) -> Option<String> {
    // Use a helper to find patterns case-insensitively
    // and return the trimmed result as a String
    pub(crate) fn find_and_extract(
        text: &str,
        pattern: &str,
        pattern_len: usize,
    ) -> Option<String> {
        let text_lower = text.to_lowercase();
        if let Some(idx) = text_lower.find(pattern) {
            let after = &text[idx + pattern_len..];
            // Trim leading whitespace and extract first word
            let cmd = after.split_whitespace().next().unwrap_or(after);
            // Trim trailing non-alphanumeric characters (except underscore)
            let cmd = cmd.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if !cmd.is_empty() {
                return Some(cmd.to_string());
            }
        }
        None
    }

    // Try patterns in order of specificity
    find_and_extract(description, "command:", 8)
        .or_else(|| find_and_extract(description, "execute:", 8))
        .or_else(|| find_and_extract(description, "execute ", 8))
}

pub(crate) fn extract_device_from_description(description: &str) -> Option<String> {
    pub(crate) fn find_and_extract(
        text: &str,
        pattern: &str,
        pattern_len: usize,
    ) -> Option<String> {
        let text_lower = text.to_lowercase();
        if let Some(idx) = text_lower.find(pattern) {
            let after = &text[idx + pattern_len..];
            let device = after.split_whitespace().next().unwrap_or(after);
            let device = device.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if !device.is_empty() {
                return Some(device.to_string());
            }
        }
        None
    }

    find_and_extract(description, "device:", 7)
        .or_else(|| find_and_extract(description, "device ", 7))
        .or_else(|| find_and_extract(description, "on ", 3))
}

/// Extract JSON string from text that may be wrapped in markdown code blocks.
/// Handles `\`\`\`json ... \`\`\`` and plain `\`\`\` ... \`\`\`` wrappers.
pub(crate) fn extract_json_from_codeblock(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.contains("```json") {
        trimmed
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .map(|s| s.trim())
    } else if trimmed.contains("```") {
        trimmed.split("```").nth(1).map(|s| s.trim())
    } else {
        None
    }
}

pub(crate) fn summarize_tool_output(data: &serde_json::Value, tool_name: &str) -> String {
    if let Some(obj) = data.as_object() {
        if let Some(count) = obj.get("count") {
            format!("{} returned {} items", tool_name, count)
        } else if let Some(msg) = obj.get("message").and_then(|m| m.as_str()) {
            format!("{}: {}", tool_name, msg)
        } else if let Some(points) = obj.get("points").and_then(|p| p.as_array()) {
            format!("{} retrieved {} data points", tool_name, points.len())
        } else if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or(id);
            format!("Retrieved: {} ({})", name, id)
        } else {
            let keys: Vec<&str> = obj.keys().take(3).map(|s| s.as_str()).collect();
            if keys.is_empty() {
                format!("{} completed", tool_name)
            } else {
                format!("{} Fields: {}", tool_name, keys.join(", "))
            }
        }
    } else {
        format!("{} completed", tool_name)
    }
}
