//! Data Push API handlers.
//!
//! POST   /api/data-push              - Create push target
//! GET    /api/data-push              - List push targets
//! GET    /api/data-push/stats        - Push statistics
//! GET    /api/data-push/:id          - Get push target
//! PUT    /api/data-push/:id          - Update push target
//! DELETE /api/data-push/:id          - Delete push target
//! POST   /api/data-push/:id/test     - Test push target
//! POST   /api/data-push/:id/start    - Start push target
//! POST   /api/data-push/:id/stop     - Stop push target
//! GET    /api/data-push/:id/logs     - List delivery logs

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;

use heramind_data_push::{CreateTargetRequest, UpdateTargetRequest};

use super::{
    common::{ok, HandlerResult},
    ServerState,
};
use crate::models::ErrorResponse;

/// Query parameters for listing push targets.
#[derive(Debug, Deserialize)]
pub struct ListTargetsQuery {
    pub enabled: Option<bool>,
}

/// Query parameters for listing delivery logs.
#[derive(Debug, Deserialize)]
pub struct ListLogsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Create a new push target.
/// POST /api/data-push
pub async fn create_push_target_handler(
    State(state): State<ServerState>,
    Json(request): Json<CreateTargetRequest>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let target = manager
        .create_target(request)
        .await
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!({
        "id": target.id,
        "name": target.name,
        "target_type": target.target_type.to_string(),
        "enabled": target.enabled,
    }))
}

/// List all push targets.
/// GET /api/data-push
pub async fn list_push_targets_handler(
    State(state): State<ServerState>,
    Query(_params): Query<ListTargetsQuery>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let targets = manager
        .list_targets()
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!({
        "targets": targets,
        "total": targets.len(),
    }))
}

/// Get a push target by ID.
/// GET /api/data-push/:id
pub async fn get_push_target_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let target = manager
        .get_target(&id)
        .map_err(|e| ErrorResponse::internal(e.to_string()))?
        .ok_or_else(|| ErrorResponse::not_found(format!("Push target not found: {}", id)))?;

    ok(json!(target))
}

/// Update a push target.
/// PUT /api/data-push/:id
pub async fn update_push_target_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateTargetRequest>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let target = manager
        .update_target(&id, request)
        .await
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!({
        "id": target.id,
        "name": target.name,
        "enabled": target.enabled,
    }))
}

/// Delete a push target.
/// DELETE /api/data-push/:id
pub async fn delete_push_target_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let deleted = manager
        .delete_target(&id)
        .await
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    if !deleted {
        return Err(ErrorResponse::not_found(format!(
            "Push target not found: {}",
            id
        )));
    }

    ok(json!({"message": "Push target deleted"}))
}

/// Test a push target by sending sample data.
/// POST /api/data-push/:id/test
pub async fn test_push_target_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let log = manager
        .test_target(&id)
        .await
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!(log))
}

/// Start a push target.
/// POST /api/data-push/:id/start
pub async fn start_push_target_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    manager
        .start_target(&id)
        .await
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!({"message": "Push target started"}))
}

/// Stop a push target.
/// POST /api/data-push/:id/stop
pub async fn stop_push_target_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    manager
        .stop_target(&id)
        .await
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!({"message": "Push target stopped"}))
}

/// List delivery logs for a push target.
/// GET /api/data-push/:id/logs
pub async fn list_delivery_logs_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Query(params): Query<ListLogsQuery>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);
    let (logs, total) = manager
        .list_delivery_logs(&id, limit, offset)
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!({
        "logs": logs,
        "total": total,
    }))
}

/// Get push statistics.
/// GET /api/data-push/stats
pub async fn get_push_stats_handler(
    State(state): State<ServerState>,
) -> HandlerResult<serde_json::Value> {
    let data_push = state.data_push.read().await;
    let manager = data_push
        .as_ref()
        .ok_or_else(|| ErrorResponse::internal("Data push manager not initialized"))?;

    let stats = manager
        .get_stats()
        .map_err(|e| ErrorResponse::internal(e.to_string()))?;

    ok(json!(stats))
}
