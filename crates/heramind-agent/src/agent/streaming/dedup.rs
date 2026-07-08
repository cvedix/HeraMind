/// Deduplicate accumulated tool results across multiple rounds.
///
/// Keeps the **latest** result for each (tool_name, key_arguments) combination.
/// When the same tool is called with the same arguments across rounds (LLM retrying),
/// only the last successful result is kept. Different arguments produce separate entries.
pub(crate) fn deduplicate_tool_results(results: &[(String, String)]) -> Vec<(String, String)> {
    // Build a key from tool name + distinguishing arguments parsed from the result JSON
    let mut seen: Vec<(String, String)> = Vec::new(); // (key, dedup_key)
    let mut deduped: Vec<(String, String)> = Vec::new();

    for (name, result) in results {
        // Create a dedup key from name + result fingerprint
        let dedup_key = make_result_dedup_key(name, result);

        if let Some(pos) = seen
            .iter()
            .position(|(k, dk)| k == name && dk == &dedup_key)
        {
            // Replace with latest result
            deduped[pos] = (name.clone(), result.clone());
        } else {
            seen.push((name.clone(), dedup_key));
            deduped.push((name.clone(), result.clone()));
        }
    }

    deduped
}

/// Create a dedup key for a tool result by extracting entity identifiers.
pub(crate) fn make_result_dedup_key(name: &str, result: &str) -> String {
    // Try to extract entity IDs from the result JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(result) {
        let mut key_parts = vec![name.to_string()];

        // Extract common entity identifiers
        for field in &["device_id", "metric", "agent_id", "rule_id", "id", "name"] {
            if let Some(val) = json.get(*field).and_then(|v| v.as_str()) {
                key_parts.push(val.to_string());
            }
        }

        // For device query results, also check nested data
        if let Some(data) = json.get("data") {
            if let Some(obj) = data.as_object() {
                for field in &["device_id", "device_name"] {
                    if let Some(val) = obj.get(*field).and_then(|v| v.as_str()) {
                        key_parts.push(val.to_string());
                    }
                }
            }
        }

        return key_parts.join("|");
    }

    // Fallback: simple hash of the result content for dedup
    let preview: String = result.chars().take(200).collect();
    let hash = preview
        .chars()
        .fold(0u64, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u64));
    format!("{}|{:016x}", name, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_keeps_latest() {
        let results = vec![
            (
                "shell".to_string(),
                r#"{"device_id":"d1","status":"ok"}"#.to_string(),
            ),
            (
                "shell".to_string(),
                r#"{"device_id":"d1","status":"updated"}"#.to_string(),
            ),
        ];
        let deduped = deduplicate_tool_results(&results);
        assert_eq!(deduped.len(), 1);
        assert!(deduped[0].1.contains("updated"));
    }

    #[test]
    fn test_dedup_different_entities_kept() {
        let results = vec![
            (
                "shell".to_string(),
                r#"{"device_id":"d1","value":1}"#.to_string(),
            ),
            (
                "shell".to_string(),
                r#"{"device_id":"d2","value":2}"#.to_string(),
            ),
        ];
        let deduped = deduplicate_tool_results(&results);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_dedup_key_json_extraction() {
        let key = make_result_dedup_key("shell", r#"{"device_id":"sensor1","status":"ok"}"#);
        assert!(key.contains("shell"));
        assert!(key.contains("sensor1"));
    }

    #[test]
    fn test_dedup_key_fallback_for_non_json() {
        let key = make_result_dedup_key("shell", "not json at all");
        assert!(key.starts_with("shell|"));
    }
}
