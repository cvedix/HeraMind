use crate::types::CliResponse;
use crate::ApiClient;
use anyhow::Result;
use serde_json::json;

/// Get current timezone setting
pub async fn get_timezone(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/settings/timezone").await?;
    Ok(CliResponse::success(data, "Timezone retrieved"))
}

/// Update timezone setting
pub async fn update_timezone(client: &ApiClient, timezone: &str) -> Result<CliResponse> {
    let body = json!({ "timezone": timezone });
    let data = client.put("/settings/timezone", &body).await?;
    Ok(CliResponse::success(data, "Timezone updated"))
}

/// List available timezones
pub async fn list_timezones(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/settings/timezones").await?;
    Ok(CliResponse::success(data, "Available timezones listed"))
}

/// Get data retention settings
pub async fn get_retention(client: &ApiClient) -> Result<CliResponse> {
    let data = client.get("/settings/retention").await?;
    Ok(CliResponse::success(data, "Retention settings retrieved"))
}

/// Update data retention settings.
///
/// Matches the backend `RetentionConfigRequest`: `enabled` and `interval_hours`
/// are required; `default_retention` and `image_retention` are optional limits
/// in hours.
pub async fn update_retention(
    client: &ApiClient,
    enabled: bool,
    interval_hours: u64,
    default_retention: Option<u64>,
    image_retention: Option<u64>,
) -> Result<CliResponse> {
    let mut body = json!({
        "enabled": enabled,
        "interval_hours": interval_hours,
    });
    if let Some(r) = default_retention {
        body["default_retention"] = json!(r);
    }
    if let Some(i) = image_retention {
        body["image_retention"] = json!(i);
    }
    let data = client.put("/settings/retention", &body).await?;
    Ok(CliResponse::success(data, "Retention settings updated"))
}

/// Trigger manual data cleanup
pub async fn trigger_cleanup(client: &ApiClient) -> Result<CliResponse> {
    let data = client.post_raw("/settings/retention/cleanup").await?;
    Ok(CliResponse::success(data, "Cleanup triggered"))
}
