//! Data Push CLI operations.

use anyhow::Result;
use serde_json::json;

use crate::api_client::extract_inner_data;
use crate::types::{BuildMeta, CliResponse};
use crate::ApiClient;

/// List push targets with compact summary.
///
/// Returns id, name, type, and enabled per target.
/// Full config is available via `heramind push-target get <id>`.
pub async fn list_targets(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/data-push").await?;
    let inner = extract_inner_data(data);

    let targets: Option<&Vec<serde_json::Value>> = inner
        .as_array()
        .or_else(|| inner.get("targets").and_then(|v| v.as_array()))
        .or_else(|| inner.get("data").and_then(|d| d.as_array()));

    let Some(targets) = targets else {
        return Ok(CliResponse::success(inner, "Push targets listed"));
    };

    let total = targets.len();
    let summary: Vec<serde_json::Value> = targets
        .iter()
        .map(|t| {
            json!({
                "id": t.get("id").and_then(|v| v.as_str()).unwrap_or("?"),
                "name": t.get("name").and_then(|v| v.as_str()).unwrap_or("(unnamed)"),
                "target_type": t.get("target_type").or_else(|| t.get("type")).and_then(|v| v.as_str()).unwrap_or("unknown"),
                "enabled": t.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            })
        })
        .collect();

    Ok(CliResponse::success(
        json!({ "total": total, "targets": summary }),
        format!("{} push target(s) listed", total),
    ))
}

/// Get a push target by ID.
pub async fn get_target(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client.get(&format!("/data-push/{}", id)).await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Push target retrieved",
    ))
}

/// Create a push target.
pub async fn create_target(
    client: &ApiClient,
    name: &str,
    target_type: &str,
    config: &str,
    schedule_type: &str,
    source_patterns: &str,
) -> Result<CliResponse> {
    // 1. Validate name is non-empty
    if name.is_empty() {
        return Ok(CliResponse::error(
            "Target name is required. Use --name <NAME>",
            "MISSING_NAME",
        ));
    }

    // 2. Validate target_type
    if target_type.is_empty() {
        return Ok(CliResponse::error_with_suggestion(
            "Target type is required. Use --type <TYPE>.",
            "MISSING_TYPE",
            "Valid types: webhook, mqtt.",
        ));
    }

    // 3. Validate config is valid JSON
    let config_val: serde_json::Value = match serde_json::from_str(config) {
        Ok(v) => v,
        Err(e) => {
            return Ok(CliResponse::error_with_suggestion(
                format!("Invalid config JSON: {}", e),
                "INVALID_JSON",
                match target_type {
                    "webhook" => "Example: --config '{\"url\":\"https://example.com/webhook\"}'",
                    "mqtt" => "Example: --config '{\"broker\":\"tcp://broker:1883\",\"topic\":\"heramind/data\"}'",
                    _ => "Provide a valid JSON object for --config.",
                },
            ));
        }
    };

    // 4. Validate target_type is known
    match target_type {
        "webhook" | "mqtt" => {}
        _ => {
            return Ok(CliResponse::error_with_suggestion(
                format!("Unknown target type '{}'.", target_type),
                "UNKNOWN_TYPE",
                "Valid types: webhook, mqtt.",
            ));
        }
    }

    let schedule = match schedule_type {
        "interval" => json!({
            "type": "interval",
            "interval_secs": 60
        }),
        _ => json!({
            "type": "event_driven",
            "event_types": ["device_metric", "extension_output"]
        }),
    };

    let body = json!({
        "name": name,
        "target_type": target_type,
        "config": config_val,
        "schedule": schedule,
        "data_filter": {
            "source_patterns": source_patterns.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>(),
            "only_changes": false
        }
    });

    let data = client.post("/data-push", &body).await?;
    let data = extract_inner_data(data);
    let target_id = data["id"].as_str().unwrap_or("unknown").to_string();

    let meta = BuildMeta {
        r#type: "push".to_string(),
        action: "create".to_string(),
        entity_id: target_id.clone(),
        entity_name: Some(name.to_string()),
        undo_command: format!("heramind push delete {}", target_id),
    };

    Ok(CliResponse::success_with_meta(
        data,
        "Push target created",
        meta,
    ))
}

/// Update a push target.
pub async fn update_target(
    client: &ApiClient,
    id: &str,
    name: Option<&str>,
    config: Option<&str>,
    enabled: Option<bool>,
) -> Result<CliResponse> {
    let mut body = json!({});
    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(c) = config {
        body["config"] = serde_json::from_str(c).unwrap_or_else(|_| json!({"url": c}));
    }
    if let Some(e) = enabled {
        body["enabled"] = json!(e);
    }

    let data = client.put(&format!("/data-push/{}", id), &body).await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Push target updated",
    ))
}

/// Delete a push target.
pub async fn delete_target(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client.delete(&format!("/data-push/{}", id)).await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Push target deleted",
    ))
}

/// Start a push target.
pub async fn start_target(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client
        .post(&format!("/data-push/{}/start", id), &json!({}))
        .await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Push target started",
    ))
}

/// Stop a push target.
pub async fn stop_target(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client
        .post(&format!("/data-push/{}/stop", id), &json!({}))
        .await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Push target stopped",
    ))
}

/// Test a push target.
pub async fn test_target(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client
        .post(&format!("/data-push/{}/test", id), &json!({}))
        .await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Push target test completed",
    ))
}

/// List delivery logs for a push target.
pub async fn list_logs(client: &ApiClient, id: &str, limit: Option<usize>) -> Result<CliResponse> {
    let path = if let Some(l) = limit {
        format!("/data-push/{}/logs?limit={}", id, l)
    } else {
        format!("/data-push/{}/logs", id)
    };
    let data = client.get(&path).await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Delivery logs listed",
    ))
}

/// Get push statistics.
pub async fn get_stats(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/data-push/stats").await?;
    Ok(CliResponse::success(
        extract_inner_data(data),
        "Push stats retrieved",
    ))
}
