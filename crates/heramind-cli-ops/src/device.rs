use crate::types::{BuildMeta, CliResponse};
use crate::ApiClient;
use anyhow::Result;
use serde_json::json;
use std::collections::BTreeMap;

/// Maximum devices before skipping metric enrichment (token budget protection).
const MAX_DEVICES_FOR_ENRICHMENT: usize = 50;
/// Maximum devices shown per type group before truncation.
const MAX_DEVICES_PER_TYPE: usize = 20;

/// List devices grouped by type with metric schema and example values.
///
/// Internal flow:
/// 1. GET /devices → device list
/// 2. Group by `device_type` field
/// 3. For each group, pick 1 online device as example
/// 4. GET /devices/{id}/current for each example (parallel)
/// 5. Build grouped response
pub async fn list_devices(
    client: &ApiClient,
    device_type: Option<&str>,
    status: Option<&str>,
) -> Result<CliResponse> {
    let mut path = "/devices?limit=100".to_string();
    if let Some(dt) = device_type {
        path.push_str(&format!("&device_type={}", dt));
    }
    if let Some(s) = status {
        path.push_str(&format!("&status={}", s));
    }
    let data = client.get(&path).await?;

    // Extract device array from API response
    let devices = extract_device_array(&data);
    let total = devices.len();

    if total == 0 {
        return Ok(CliResponse::success(
            json!({"summary": {"total": 0, "online": 0, "offline": 0, "type_count": 0}, "types": [], "ungrouped": []}),
            "No devices found",
        ));
    }

    // Count online/offline
    let online_count = devices
        .iter()
        .filter(|d| {
            d.get("status").and_then(|v| v.as_str()) == Some("online")
                || d.get("online").and_then(|v| v.as_bool()) == Some(true)
        })
        .count();
    let offline_count = total - online_count;

    // Group by device_type
    let mut typed_groups: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    let mut ungrouped_devices: Vec<serde_json::Value> = Vec::new();

    for device in &devices {
        match device
            .get("device_type")
            .or_else(|| device.get("type"))
            .and_then(|v| v.as_str())
        {
            Some(dt) if !dt.is_empty() => {
                typed_groups
                    .entry(dt.to_string())
                    .or_default()
                    .push(device.clone());
            }
            _ => {
                ungrouped_devices.push(device.clone());
            }
        }
    }

    // Enrich with example metrics if within budget
    let enrich = total <= MAX_DEVICES_FOR_ENRICHMENT;

    // Build type groups
    let mut types_response = Vec::new();
    if enrich {
        // Collect example device IDs for parallel fetch
        let example_ids: Vec<(String, String)> = typed_groups
            .iter()
            .filter_map(|(type_name, devs)| {
                // Pick first online device as example
                let example = devs
                    .iter()
                    .find(|d| d.get("status").and_then(|v| v.as_str()) == Some("online"))
                    .or_else(|| devs.first())?;
                let id = extract_device_id(example)?;
                Some((type_name.clone(), id))
            })
            .collect();

        // Fetch example metrics (sequential, types are few)
        let example_results = fetch_examples(client, &example_ids).await;

        for (type_name, devs) in &typed_groups {
            let group_online = devs
                .iter()
                .filter(|d| d.get("status").and_then(|v| v.as_str()) == Some("online"))
                .count();

            let mut type_entry = json!({
                "type": type_name,
                "metric_fields": [],
                "online": group_online,
                "offline": devs.len() - group_online,
            });

            // Add example data if available
            if let Some(example_id) = example_ids
                .iter()
                .find(|(t, _)| t == type_name)
                .map(|(_, id)| id.as_str())
            {
                if let Some(metrics) = example_results.get(example_id) {
                    let (field_names, command_names, example_obj) =
                        build_example(example_id, metrics, devs);
                    type_entry["metric_fields"] = field_names;
                    type_entry["command_fields"] = command_names;
                    type_entry["example"] = example_obj;
                }
            }

            // Device list for this type
            type_entry["devices"] = build_device_list(devs);
            types_response.push(type_entry);
        }
    } else {
        // Too many devices — skip enrichment, just return groups
        for (type_name, devs) in &typed_groups {
            let group_online = devs
                .iter()
                .filter(|d| d.get("status").and_then(|v| v.as_str()) == Some("online"))
                .count();
            types_response.push(json!({
                "type": type_name,
                "metric_fields": [],
                "online": group_online,
                "offline": devs.len() - group_online,
                "example": null,
                "devices": build_device_list(devs),
            }));
        }
    }

    // Build ungrouped entries
    let ungrouped_response: Vec<serde_json::Value> = if enrich && !ungrouped_devices.is_empty() {
        let ungrouped_ids: Vec<(String, String)> = ungrouped_devices
            .iter()
            .filter_map(|d| extract_device_id(d).map(|id| ("_ungrouped".to_string(), id)))
            .collect();
        let ungrouped_results: BTreeMap<String, serde_json::Value> =
            fetch_examples(client, &ungrouped_ids).await;

        ungrouped_devices
            .iter()
            .map(|d| {
                let id = extract_device_id(d).unwrap_or_default();
                let name = d.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let dev_status = d
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let mut entry = json!({
                    "id": id,
                    "name": name,
                    "status": dev_status,
                });
                if let Some(metrics) = ungrouped_results.get(&id) {
                    entry["metrics"] = compact_metric_values(metrics);
                }
                entry
            })
            .collect()
    } else {
        ungrouped_devices
            .iter()
            .map(|d| {
                let id = extract_device_id(d).unwrap_or_default();
                let name = d.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let dev_status = d
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                json!({"id": id, "name": name, "status": dev_status})
            })
            .collect()
    };

    let response = json!({
        "summary": {
            "total": total,
            "online": online_count,
            "offline": offline_count,
            "type_count": typed_groups.len(),
        },
        "types": types_response,
        "ungrouped": ungrouped_response,
    });

    Ok(CliResponse::success(response, "Devices listed"))
}

/// Fetch example metrics for multiple devices sequentially.
/// Types are typically 2-5, so sequential is fast enough.
async fn fetch_examples(
    client: &ApiClient,
    ids: &[(String, String)],
) -> BTreeMap<String, serde_json::Value> {
    let mut map = BTreeMap::new();
    for (_, id) in ids {
        match client.get(&format!("/devices/{}/current", id)).await {
            Ok(data) => {
                map.insert(id.clone(), data);
            }
            Err(_) => continue,
        }
    }
    map
}

/// Sanitize a metric value for LLM consumption.
/// Truncates long strings (likely base64 binary data) to avoid wasting tokens.
fn sanitize_metric_value(val: &serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::String(s) => {
            if s.len() > 80 {
                // Truncate and mark as binary/large — LLM doesn't need the full payload
                let prefix = &s[..s.floor_char_boundary(60)];
                json!(format!(
                    "{}... <truncated, {} bytes total>",
                    prefix,
                    s.len()
                ))
            } else {
                val.clone()
            }
        }
        _ => val.clone(),
    }
}

/// Sanitize the full /devices/{id}/current response to truncate binary metric values.
fn sanitize_device_current(data: &serde_json::Value) -> serde_json::Value {
    let mut result = data.clone();
    // Navigate to data.metrics and sanitize each metric's value
    let metrics = result.pointer_mut("/data/metrics");
    if let Some(m) = metrics.and_then(|v| v.as_object_mut()) {
        for (_name, info) in m.iter_mut() {
            if let Some(obj) = info.as_object_mut() {
                if let Some(val) = obj.get_mut("value") {
                    *val = sanitize_metric_value(val);
                }
            }
        }
    }
    result
}

/// Extract device array from API response (handles multiple response shapes).
fn extract_device_array(data: &serde_json::Value) -> Vec<serde_json::Value> {
    data.get("data")
        .and_then(|d| d.get("devices"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default()
}

/// Extract device ID from a device object.
fn extract_device_id(device: &serde_json::Value) -> Option<String> {
    device
        .get("device_id")
        .or_else(|| device.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Build metric field names, command names, and example object from /current response.
fn build_example(
    example_id: &str,
    current_data: &serde_json::Value,
    devs: &[serde_json::Value],
) -> (serde_json::Value, serde_json::Value, serde_json::Value) {
    let response_data = current_data.get("data").unwrap_or(current_data);
    let metrics = response_data.get("metrics");
    let commands = response_data.get("commands");

    let mut field_names = Vec::new();
    let mut example_values = serde_json::Map::new();

    let example_name = devs
        .iter()
        .find(|d| extract_device_id(d).as_deref() == Some(example_id))
        .and_then(|d| {
            d.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();

    // Extract metric field names + example values
    if let Some(metrics_obj) = metrics.and_then(|m| m.as_object()) {
        for (name, info) in metrics_obj {
            if let Some(val) = info.get("value") {
                if !val.is_null() {
                    field_names.push(json!(name));
                    example_values.insert(name.clone(), sanitize_metric_value(val));
                }
            }
        }
    }

    // Extract command names
    let command_names: Vec<serde_json::Value> = commands
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|cmd| cmd.get("name").and_then(|n| n.as_str()).map(|s| json!(s)))
                .collect()
        })
        .unwrap_or_default();

    example_values.insert("id".to_string(), json!(example_id));
    example_values.insert("name".to_string(), json!(example_name));

    (
        json!(field_names),
        json!(command_names),
        json!(example_values),
    )
}

/// Extract compact metric values from /current response (for ungrouped devices).
fn compact_metric_values(current_data: &serde_json::Value) -> serde_json::Value {
    let metrics = current_data
        .get("data")
        .and_then(|d| d.get("metrics"))
        .or_else(|| current_data.get("metrics"));

    if let Some(metrics_obj) = metrics.and_then(|m| m.as_object()) {
        let mut values = serde_json::Map::new();
        for (name, info) in metrics_obj {
            if let Some(val) = info.get("value") {
                if !val.is_null() {
                    values.insert(name.clone(), sanitize_metric_value(val));
                }
            }
        }
        json!(values)
    } else {
        json!({})
    }
}

/// Build device list for a type group, truncated to MAX_DEVICES_PER_TYPE.
fn build_device_list(devs: &[serde_json::Value]) -> serde_json::Value {
    let total = devs.len();
    let list: Vec<serde_json::Value> = devs
        .iter()
        .take(MAX_DEVICES_PER_TYPE)
        .map(|d| {
            let id = extract_device_id(d).unwrap_or_default();
            let name = d.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let dev_status = d
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            json!({"id": id, "name": name, "status": dev_status})
        })
        .collect();

    let mut result = json!({ "list": list });
    if total > MAX_DEVICES_PER_TYPE {
        result["total"] = json!(total);
        result["truncated"] = json!(true);
    }
    result
}

/// Get device details (metadata + metrics + commands) via /current endpoint.
pub async fn get_device(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client.get(&format!("/devices/{}/current", id)).await?;
    let sanitized = sanitize_device_current(&data);
    Ok(CliResponse::success(sanitized, "Device details retrieved"))
}

/// Create a new device
pub async fn create_device(
    client: &ApiClient,
    name: &str,
    device_type: &str,
    adapter_type: &str,
    device_id: Option<&str>,
    connection_config: Option<serde_json::Value>,
) -> Result<CliResponse> {
    let mut body = json!({
        "name": name,
        "device_type": device_type,
        "adapter_type": adapter_type,
        "connection_config": {}
    });
    if let Some(id) = device_id {
        // Only forward non-empty strings — clap passes empty string when the
        // user explicitly sets `--id ""`, treat that as "auto-generate".
        if !id.is_empty() {
            body["device_id"] = json!(id);
        }
    }
    if let Some(config) = connection_config {
        body["connection_config"] = config;
    }

    let data = client.post("/devices", &body).await?;
    let device_id = data
        .get("data")
        .and_then(|d| d.get("device_id"))
        .and_then(|v| v.as_str())
        .or_else(|| data["id"].as_str())
        .unwrap_or("unknown")
        .to_string();

    let meta = BuildMeta {
        r#type: "device".to_string(),
        action: "create".to_string(),
        entity_id: device_id.clone(),
        entity_name: Some(name.to_string()),
        undo_command: format!("heramind device delete {}", device_id),
    };

    Ok(CliResponse::success_with_meta(data, "Device created", meta))
}

/// Update device
pub async fn update_device(
    client: &ApiClient,
    id: &str,
    name: Option<&str>,
    connection_config: Option<serde_json::Value>,
) -> Result<CliResponse> {
    let mut body = json!({});
    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(config) = connection_config {
        body["connection_config"] = config;
    }

    let data = client.put(&format!("/devices/{}", id), &body).await?;
    Ok(CliResponse::success(data, "Device updated"))
}

/// Delete device
pub async fn delete_device(client: &ApiClient, id: &str) -> Result<CliResponse> {
    client.delete(&format!("/devices/{}", id)).await?;
    Ok(CliResponse::success(json!({ "id": id }), "Device deleted"))
}

/// Get latest metrics for a device
pub async fn get_latest_metrics(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client.get(&format!("/devices/{}/current", id)).await?;
    let sanitized = sanitize_device_current(&data);
    Ok(CliResponse::success(sanitized, "Latest metrics retrieved"))
}

/// Get historical telemetry data for a device
pub async fn get_telemetry_history(
    client: &ApiClient,
    id: &str,
    metric: Option<&str>,
    time_range: Option<&str>,
    compress: bool,
) -> Result<CliResponse> {
    let mut path = format!("/devices/{}/telemetry", id);
    let mut params = Vec::new();
    if let Some(m) = metric {
        params.push(format!("metric={}", m));
    }
    // Parse --time-range (e.g., "1h", "24h", "7d", "30d") to Unix seconds and set start
    if let Some(tr) = time_range {
        let end = chrono::Utc::now().timestamp();
        let start = parse_time_range_to_timestamp(tr, end).unwrap_or(end - 86400);
        params.push(format!("start={}", start));
        params.push(format!("end={}", end));
    }
    if compress {
        params.push("compress=true".to_string());
    }
    if !params.is_empty() {
        path.push('?');
        path.push_str(&params.join("&"));
    }

    let data = client.get(&path).await?;
    Ok(CliResponse::success(data, "Telemetry history retrieved"))
}

/// Parse a human-readable time range string (e.g., "1h", "24h", "7d", "30d") to a start timestamp.
fn parse_time_range_to_timestamp(range: &str, now_ts: i64) -> Option<i64> {
    let range = range.trim();
    if range.is_empty() {
        return None;
    }
    // Extract number suffix: last char(s)
    let num_end = range.len()
        - range
            .chars()
            .last()
            .map_or(0, |c| if c.is_ascii_alphabetic() { 1 } else { 0 });
    let num: i64 = range[..num_end].parse().ok()?;
    let unit = &range[num_end..];
    let secs = match unit {
        "s" => num,
        "m" | "min" | "mins" => num * 60,
        "h" | "hr" | "hrs" => num * 3600,
        "d" | "day" | "days" => num * 86400,
        "w" | "wk" | "wks" => num * 7 * 86400,
        "mo" | "month" | "months" => num * 30 * 86400,
        _ => return None,
    };
    Some(now_ts - secs)
}

/// Send control command to a device
pub async fn control_device(
    client: &ApiClient,
    id: &str,
    command: &str,
    params: serde_json::Value,
) -> Result<CliResponse> {
    let body = json!({ "params": params });
    let data = client
        .post(&format!("/devices/{}/command/{}", id, command), &body)
        .await?;
    Ok(CliResponse::success(data, "Command sent"))
}

/// List device types
pub async fn list_device_types(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/device-types").await?;
    Ok(CliResponse::success(data, "Device types listed"))
}

/// Get device type by ID
pub async fn get_device_type(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client.get(&format!("/device-types/{}", id)).await?;
    Ok(CliResponse::success(data, "Device type retrieved"))
}

/// Convert a display name to a snake_case device type ID.
/// E.g., "温湿度传感器" -> "wen_shi_du_chuan_gan_qi", "TempSensor" -> "temp_sensor"
fn name_to_type_id(name: &str) -> String {
    // Use a simple approach: lowercase, replace non-alphanumeric with underscore, collapse
    let id: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    // Collapse multiple underscores
    let mut result = String::new();
    let mut prev_underscore = false;
    for ch in id.chars() {
        if ch == '_' {
            if !prev_underscore {
                result.push(ch);
            }
            prev_underscore = true;
        } else {
            result.push(ch);
            prev_underscore = false;
        }
    }
    // Trim leading/trailing underscores
    result.trim_matches('_').to_string()
}

/// Create a new device type
pub async fn create_device_type(
    client: &ApiClient,
    id: Option<&str>,
    name: &str,
    metrics: serde_json::Value,
    commands: Option<serde_json::Value>,
) -> Result<CliResponse> {
    let device_type = id
        .map(|s| s.to_string())
        .unwrap_or_else(|| name_to_type_id(name));

    let mut body = json!({
        "device_type": device_type,
        "name": name,
        "description": "",
        "categories": [],
        "mode": "simple",
        "metrics": metrics,
        "uplink_samples": [],
        "parameters": [],
        "commands": [],
    });
    if let Some(cmds) = commands {
        body["commands"] = cmds;
    }

    let data = client.post("/device-types", &body).await?;
    let meta = BuildMeta {
        r#type: "device_type".to_string(),
        action: "create".to_string(),
        entity_id: device_type.clone(),
        entity_name: Some(name.to_string()),
        undo_command: format!("heramind device types delete {}", device_type),
    };

    Ok(CliResponse::success_with_meta(
        data,
        "Device type created",
        meta,
    ))
}

/// Delete device type
pub async fn delete_device_type(client: &ApiClient, id: &str) -> Result<CliResponse> {
    client.delete(&format!("/device-types/{}", id)).await?;
    Ok(CliResponse::success(
        json!({ "id": id }),
        "Device type deleted",
    ))
}

/// List pending device drafts (auto-discovered devices awaiting approval)
pub async fn list_drafts(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/devices/drafts").await?;
    Ok(CliResponse::success(data, "Device drafts listed"))
}

/// Get a specific device draft
pub async fn get_draft(client: &ApiClient, device_id: &str) -> Result<CliResponse> {
    let data = client
        .get(&format!("/devices/drafts/{}", device_id))
        .await?;
    Ok(CliResponse::success(data, "Device draft retrieved"))
}

/// Approve a device draft
pub async fn approve_draft(
    client: &ApiClient,
    device_id: &str,
    name: Option<&str>,
    device_type: Option<&str>,
) -> Result<CliResponse> {
    let mut body = json!({});
    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(t) = device_type {
        body["device_type"] = json!(t);
    }
    let data = client
        .post(&format!("/devices/drafts/{}/approve", device_id), &body)
        .await?;
    Ok(CliResponse::success(data, "Device draft approved"))
}

/// Reject a device draft
pub async fn reject_draft(client: &ApiClient, device_id: &str) -> Result<CliResponse> {
    client
        .post(&format!("/devices/drafts/{}/reject", device_id), &json!({}))
        .await?;
    Ok(CliResponse::success(
        json!({ "id": device_id }),
        "Device draft rejected",
    ))
}

/// Get auto-discovery configuration
pub async fn get_onboard_config(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/devices/drafts/config").await?;
    Ok(CliResponse::success(data, "Onboard config retrieved"))
}

/// Update auto-discovery configuration
pub async fn update_onboard_config(
    client: &ApiClient,
    enabled: Option<bool>,
    max_samples: Option<u32>,
    auto_approve: Option<bool>,
) -> Result<CliResponse> {
    let mut body = json!({});
    if let Some(e) = enabled {
        body["enabled"] = json!(e);
    }
    if let Some(m) = max_samples {
        body["max_samples"] = json!(m);
    }
    if let Some(a) = auto_approve {
        body["auto_approve"] = json!(a);
    }
    let data = client.put("/devices/drafts/config", &body).await?;
    Ok(CliResponse::success(data, "Onboard config updated"))
}

/// Write metric data point for a device
pub async fn write_metric(
    client: &ApiClient,
    id: &str,
    metric: &str,
    value: serde_json::Value,
    timestamp: Option<i64>,
) -> Result<CliResponse> {
    let mut body = json!({
        "metric": metric,
        "value": value,
    });
    if let Some(ts) = timestamp {
        body["timestamp"] = json!(ts);
    }
    let data = client
        .post(&format!("/devices/{}/metrics", id), &body)
        .await?;
    Ok(CliResponse::success(data, "Metric written"))
}

/// Get webhook URL for a device
pub async fn get_webhook_url(client: &ApiClient, id: &str) -> Result<CliResponse> {
    let data = client.get(&format!("/devices/{}/webhook-url", id)).await?;
    Ok(CliResponse::success(data, "Webhook URL retrieved"))
}
