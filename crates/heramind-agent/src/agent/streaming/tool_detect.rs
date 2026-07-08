/// Detect JSON tool calls in buffer.
///
/// Looks for JSON array format: [{"name": "tool", "arguments": {...}}, ...]
/// Returns Some((start_pos, json_text, remaining_buffer)) if found, None otherwise.
pub(crate) fn detect_json_tool_calls(buffer: &str) -> Option<(usize, String, String)> {
    // Find the first '[' that might start a JSON array
    let start = buffer.find('[')?;

    // Find the matching closing ']' while properly handling:
    // 1. String literals (skip brackets inside "...")
    // 2. Escape sequences (skip escaped characters like \")
    let chars: Vec<char> = buffer[start..].chars().collect();
    let mut bracket_count = 0isize;
    let mut in_string = false;
    let mut end = None;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if in_string {
            if c == '\\' {
                // Skip escaped character
                i += 2;
                continue;
            } else if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '"' => in_string = true,
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        // Calculate byte offset from char index
                        let byte_offset: usize = chars[..=i].iter().map(|c| c.len_utf8()).sum();
                        end = Some(start + byte_offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }

    let end = end?;

    // Extract the JSON array
    let json_str = buffer[start..end].to_string();

    // Check if it looks like a tool call (has "name", "tool", or "function" key)
    if !json_str.contains("\"name\"")
        && !json_str.contains("\"tool\"")
        && !json_str.contains("\"function\"")
    {
        return None;
    }

    // Verify it's valid JSON
    let json_value = serde_json::from_str::<serde_json::Value>(&json_str).ok()?;

    // Validate that at least one element has a valid string "name" field
    // This prevents false positives from malformed JSON like [{"name":"[...]"}]
    if let Some(arr) = json_value.as_array() {
        let has_valid_tool_call = arr.iter().any(|item| {
            if let Some(obj) = item.as_object() {
                // Check if "name", "tool", or "function" field exists and is a valid string
                let name_value = obj
                    .get("name")
                    .or_else(|| obj.get("tool"))
                    .or_else(|| obj.get("function"));

                if let Some(name) = name_value {
                    if let Some(name_str) = name.as_str() {
                        // Ensure the name is a simple string (not a JSON string containing nested JSON)
                        // A valid tool name should not start with '[' or '{'
                        let trimmed = name_str.trim();
                        return !trimmed.starts_with('[') && !trimmed.starts_with('{');
                    }
                }
            }
            false
        });

        if !has_valid_tool_call {
            return None;
        }
    } else {
        return None;
    }

    // Return start position, the JSON, and remaining buffer
    let remaining = buffer[end..].to_string();
    Some((start, json_str, remaining))
}
