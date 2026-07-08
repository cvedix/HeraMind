//! Dashboard handlers
//!
//! Provides API endpoints for managing visual dashboards with components.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{
    common::{ok, HandlerResult},
    ServerState,
};
use crate::models::ErrorResponse;
use heramind_core::event::HeraMindEvent;
use heramind_storage::dashboards::{
    default_templates, Dashboard as StoredDashboard, DashboardComponent as StoredComponent,
    DashboardLayout as StoredLayout, DashboardTemplate as StoredTemplate,
    SharePermissions as StoredSharePermissions, ShareToken as StoredShareToken,
};

/// Emit a DashboardUpdated event to notify frontend of changes.
fn emit_dashboard_event(state: &ServerState, dashboard_id: &str, action: &str) {
    if let Some(event_bus) = state.core.event_bus.clone() {
        let event = HeraMindEvent::DashboardUpdated {
            dashboard_id: dashboard_id.to_string(),
            action: action.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };
        tokio::spawn(async move {
            event_bus.publish(event).await;
        });
    }
}

// ============================================================================
// API Types (match frontend expectations)
// ============================================================================

/// Dashboard layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub columns: u32,
    #[serde(alias = "rows", rename = "rows")]
    pub rows: RowsValue,
    pub breakpoints: LayoutBreakpoints,
}

impl Default for DashboardLayout {
    fn default() -> Self {
        Self {
            columns: 12,
            rows: RowsValue::String("auto".to_string()),
            breakpoints: LayoutBreakpoints::default(),
        }
    }
}

/// Rows value - can be "auto" string or a number
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RowsValue {
    String(String),
    Number(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBreakpoints {
    pub lg: u32,
    pub md: u32,
    pub sm: u32,
    pub xs: u32,
}

impl Default for LayoutBreakpoints {
    fn default() -> Self {
        Self {
            lg: 1200,
            md: 996,
            sm: 768,
            xs: 480,
        }
    }
}

/// Component position on the grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentPosition {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_w: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_h: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_w: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_h: Option<u32>,
}

/// Dashboard component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardComponent {
    pub id: String,
    #[serde(alias = "type", rename = "type")]
    pub component_type: String,
    pub position: ComponentPosition,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "title",
        rename = "title"
    )]
    pub title: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "data_source",
        rename = "data_source"
    )]
    pub data_source: Option<JsonValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "display",
        rename = "display"
    )]
    pub display: Option<JsonValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "config",
        rename = "config"
    )]
    pub config: Option<JsonValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "actions",
        rename = "actions"
    )]
    pub actions: Option<JsonValue>,
}

/// Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub layout: DashboardLayout,
    pub components: Vec<DashboardComponent>,
    #[serde(alias = "created_at", rename = "created_at")]
    pub created_at: i64,
    #[serde(alias = "updated_at", rename = "updated_at")]
    pub updated_at: i64,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "is_default",
        rename = "is_default"
    )]
    pub is_default: Option<bool>,
    /// Manual sort order (lower = higher in the sidebar list).
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "sort_order",
        rename = "sort_order"
    )]
    pub sort_order: Option<i32>,
}

/// Request to create a dashboard
#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    #[serde(default)]
    pub layout: DashboardLayout,
    #[serde(default)]
    pub components: Vec<CreateDashboardComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardComponent {
    /// Optional client-provided ID; if absent, server generates one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(alias = "type", rename = "type")]
    pub component_type: String,
    pub position: ComponentPosition,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "title",
        rename = "title"
    )]
    pub title: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "data_source",
        rename = "data_source"
    )]
    pub data_source: Option<JsonValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "display",
        rename = "display"
    )]
    pub display: Option<JsonValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "config",
        rename = "config"
    )]
    pub config: Option<JsonValue>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "actions",
        rename = "actions"
    )]
    pub actions: Option<JsonValue>,
}

/// Request to update a dashboard - use JsonValue to accept flexible formats
#[derive(Debug, Deserialize)]
pub struct UpdateDashboardRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<DashboardLayout>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<JsonValue>>,
}

/// Dashboard template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub layout: DashboardLayout,
    pub components: Vec<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_resources: Option<RequiredResources>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<u32>,
}

/// Response with dashboards list
#[derive(Serialize)]
pub struct DashboardsResponse {
    pub dashboards: Vec<Dashboard>,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

// ============================================================================
// Conversion Helpers
// ============================================================================

/// Convert stored dashboard to API dashboard
fn stored_to_api(dashboard: &StoredDashboard) -> Dashboard {
    Dashboard {
        id: dashboard.id.clone(),
        name: dashboard.name.clone(),
        layout: convert_layout(&dashboard.layout),
        components: dashboard.components.iter().map(convert_component).collect(),
        created_at: dashboard.created_at,
        updated_at: dashboard.updated_at,
        is_default: dashboard.is_default,
        sort_order: dashboard.sort_order,
    }
}

/// Convert API layout to stored layout
fn api_to_stored_layout(layout: &DashboardLayout) -> StoredLayout {
    StoredLayout {
        columns: layout.columns,
        rows: match &layout.rows {
            RowsValue::String(s) => heramind_storage::dashboards::RowsValue::String(s.clone()),
            RowsValue::Number(n) => heramind_storage::dashboards::RowsValue::Number(*n),
        },
        breakpoints: heramind_storage::dashboards::LayoutBreakpoints {
            lg: layout.breakpoints.lg,
            md: layout.breakpoints.md,
            sm: layout.breakpoints.sm,
            xs: layout.breakpoints.xs,
        },
    }
}

/// Convert stored layout to API layout
fn convert_layout(layout: &StoredLayout) -> DashboardLayout {
    DashboardLayout {
        columns: layout.columns,
        rows: match &layout.rows {
            heramind_storage::dashboards::RowsValue::String(s) => RowsValue::String(s.clone()),
            heramind_storage::dashboards::RowsValue::Number(n) => RowsValue::Number(*n),
        },
        breakpoints: LayoutBreakpoints {
            lg: layout.breakpoints.lg,
            md: layout.breakpoints.md,
            sm: layout.breakpoints.sm,
            xs: layout.breakpoints.xs,
        },
    }
}

/// Convert API component to stored component.
/// Preserves client-provided ID when available, otherwise caller must assign one.
fn api_to_stored_component(component: &CreateDashboardComponent) -> StoredComponent {
    StoredComponent {
        id: component.id.clone().unwrap_or_default(),
        component_type: component.component_type.clone(),
        position: heramind_storage::dashboards::ComponentPosition {
            x: component.position.x,
            y: component.position.y,
            w: component.position.w,
            h: component.position.h,
            min_w: component.position.min_w,
            min_h: component.position.min_h,
            max_w: component.position.max_w,
            max_h: component.position.max_h,
        },
        title: component.title.clone(),
        data_source: component.data_source.clone(),
        display: component.display.clone(),
        config: component.config.clone(),
        actions: component.actions.clone(),
    }
}

/// Convert stored component to API component
fn convert_component(component: &StoredComponent) -> DashboardComponent {
    DashboardComponent {
        id: component.id.clone(),
        component_type: component.component_type.clone(),
        position: ComponentPosition {
            x: component.position.x,
            y: component.position.y,
            w: component.position.w,
            h: component.position.h,
            min_w: component.position.min_w,
            min_h: component.position.min_h,
            max_w: component.position.max_w,
            max_h: component.position.max_h,
        },
        title: component.title.clone(),
        data_source: component.data_source.clone(),
        display: component.display.clone(),
        config: component.config.clone(),
        actions: component.actions.clone(),
    }
}

/// Convert stored template to API template
fn stored_template_to_api(template: &StoredTemplate) -> DashboardTemplate {
    DashboardTemplate {
        id: template.id.clone(),
        name: template.name.clone(),
        description: template.description.clone(),
        category: template.category.clone(),
        icon: template.icon.clone(),
        layout: convert_layout(&template.layout),
        components: template.components.clone(),
        required_resources: template
            .required_resources
            .as_ref()
            .map(|r| RequiredResources {
                devices: r.devices,
                agents: r.agents,
                rules: r.rules,
            }),
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// List all dashboards
///
/// Performance optimization: Supports pagination via limit/offset query parameters.
/// Example: GET /api/dashboards?limit=10&offset=20
pub async fn list_dashboards_handler(
    State(state): State<ServerState>,
    Query(params): Query<PaginationParams>,
) -> HandlerResult<DashboardsResponse> {
    // Enforce reasonable limits to prevent performance issues
    let limit = params.limit.unwrap_or(100).min(1000); // Default 100, max 1000
    let offset = params.offset.unwrap_or(0);

    // Get total count for pagination metadata (only when paginating)
    let total = if limit < usize::MAX || offset > 0 {
        state.dashboard_store.count().ok()
    } else {
        None
    };

    let dashboards = state
        .dashboard_store
        .list_paginated(Some(limit), Some(offset))
        .map_err(|e| ErrorResponse::internal(format!("Failed to list dashboards: {}", e)))?;

    // Sort by `sort_order` so the sidebar reflects manual reordering. Legacy
    // rows without `sort_order` fall to the bottom (i32::MAX); stable sort
    // keeps their relative order intact.
    let mut dashboards = dashboards;
    dashboards.sort_by_key(|d| d.sort_order.unwrap_or(i32::MAX));

    let api_dashboards: Vec<Dashboard> = dashboards.iter().map(stored_to_api).collect();
    let count = api_dashboards.len();

    ok(DashboardsResponse {
        dashboards: api_dashboards,
        count,
        total,
        limit: if total.is_some() { Some(limit) } else { None },
        offset: if total.is_some() { Some(offset) } else { None },
    })
}

/// Get a dashboard by ID
pub async fn get_dashboard_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<Dashboard> {
    // Special handling for "overview" and "blank" template IDs
    if id == "overview" || id == "blank" {
        let templates = default_templates();
        let template = templates
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| ErrorResponse::not_found(format!("Template '{}' not found", id)))?;

        let now = chrono::Utc::now().timestamp();
        return ok(Dashboard {
            id: template.id.clone(),
            name: template.name.clone(),
            layout: convert_layout(&template.layout),
            components: vec![],
            created_at: now,
            updated_at: now,
            is_default: Some(id == "overview"),
            sort_order: None,
        });
    }

    let dashboard = state
        .dashboard_store
        .load(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load dashboard: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found(format!("Dashboard '{}' not found", id)))?;

    ok(stored_to_api(&dashboard))
}

/// Create a new dashboard
pub async fn create_dashboard_handler(
    State(state): State<ServerState>,
    Json(req): Json<CreateDashboardRequest>,
) -> HandlerResult<Dashboard> {
    let id = format!("dashboard_{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().timestamp();

    let next_order = state
        .dashboard_store
        .max_sort_order()
        .map_err(|e| ErrorResponse::internal(format!("Failed to compute sort order: {}", e)))?
        + 1;

    let stored_dashboard = StoredDashboard {
        id: id.clone(),
        name: req.name,
        layout: api_to_stored_layout(&req.layout),
        components: req
            .components
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let mut comp = api_to_stored_component(c);
                if comp.id.is_empty() {
                    comp.id = format!("component_{}", i);
                }
                comp
            })
            .collect(),
        created_at: now,
        updated_at: now,
        is_default: None,
        sort_order: Some(next_order),
    };

    state
        .dashboard_store
        .save(&stored_dashboard)
        .map_err(|e| ErrorResponse::internal(format!("Failed to save dashboard: {}", e)))?;

    emit_dashboard_event(&state, &stored_dashboard.id, "create");

    ok(stored_to_api(&stored_dashboard))
}

/// Update a dashboard
pub async fn update_dashboard_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDashboardRequest>,
) -> HandlerResult<Dashboard> {
    let mut dashboard = state
        .dashboard_store
        .load(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load dashboard: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found(format!("Dashboard '{}' not found", id)))?;

    // Update fields if provided
    if let Some(name) = req.name {
        dashboard.name = name;
    }
    if let Some(layout) = req.layout {
        dashboard.layout = api_to_stored_layout(&layout);
    }
    if let Some(components) = req.components {
        // Parse components from JSON — fail if any component is invalid
        let parsed: Result<Vec<StoredComponent>, String> = components
            .iter()
            .enumerate()
            .map(|(i, c)| {
                serde_json::from_value::<StoredComponent>(c.clone())
                    .map_err(|e| format!("component[{}]: {}", i, e))
            })
            .collect();
        match parsed {
            Ok(parsed_components) => dashboard.components = parsed_components,
            Err(e) => {
                return Err(
                    ErrorResponse::bad_request(format!("Invalid component data: {}", e)).with_hint(
                        "Each component needs: id, type, title, position {x,y,w,h}.\n\
                             Use 'heramind dashboard add-components' instead of full replacement.",
                    ),
                )
            }
        }
    }
    dashboard.updated_at = chrono::Utc::now().timestamp();

    state
        .dashboard_store
        .save(&dashboard)
        .map_err(|e| ErrorResponse::internal(format!("Failed to save dashboard: {}", e)))?;

    emit_dashboard_event(&state, &id, "update");

    ok(stored_to_api(&dashboard))
}

/// Add components to a dashboard (append mode)
pub async fn add_components_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(req): Json<AddComponentsRequest>,
) -> HandlerResult<Dashboard> {
    let mut dashboard = state
        .dashboard_store
        .load(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load dashboard: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found(format!("Dashboard '{}' not found", id)))?;

    // Parse and append new components
    let new_components: Vec<StoredComponent> = req
        .components
        .iter()
        .enumerate()
        .map(|(i, c)| {
            serde_json::from_value::<StoredComponent>(c.clone()).map_err(|e| {
                format!("component[{}]: {}", i, e)
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            ErrorResponse::bad_request(format!("Invalid component data: {}", e)).with_hint(
                "Each component needs: id (string), type (value-card|line-chart|bar-chart|gauge|markdown-display), title (string), \
                 position {x, y, w, h}.\n\
                 Example: {\"id\":\"temp\",\"type\":\"value-card\",\"title\":\"Temp\",\"position\":{\"x\":0,\"y\":0,\"w\":4,\"h\":2},\
                 \"data_source\":{\"type\":\"device\",\"sourceId\":\"<device-id>\",\"property\":\"<metric>\"}}",
            )
        })?;

    dashboard.components.extend(new_components);
    dashboard.updated_at = chrono::Utc::now().timestamp();

    state
        .dashboard_store
        .save(&dashboard)
        .map_err(|e| ErrorResponse::internal(format!("Failed to save dashboard: {}", e)))?;

    emit_dashboard_event(&state, &id, "add_components");

    ok(stored_to_api(&dashboard))
}

/// Remove components from a dashboard by ID
pub async fn remove_components_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(req): Json<RemoveComponentsRequest>,
) -> HandlerResult<serde_json::Value> {
    let mut dashboard = state
        .dashboard_store
        .load(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load dashboard: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found(format!("Dashboard '{}' not found", id)))?;

    let before = dashboard.components.len();
    dashboard.components.retain(|c| !req.ids.contains(&c.id));
    let removed = before - dashboard.components.len();
    dashboard.updated_at = chrono::Utc::now().timestamp();

    state
        .dashboard_store
        .save(&dashboard)
        .map_err(|e| ErrorResponse::internal(format!("Failed to save dashboard: {}", e)))?;

    emit_dashboard_event(&state, &id, "remove_components");

    ok(serde_json::json!({
        "ok": true,
        "removed": removed,
        "remaining": dashboard.components.len(),
    }))
}

/// Request to add components
#[derive(Debug, Deserialize)]
pub struct AddComponentsRequest {
    pub components: Vec<JsonValue>,
}

/// Request to remove components by ID
#[derive(Debug, Deserialize)]
pub struct RemoveComponentsRequest {
    pub ids: Vec<String>,
}

/// Delete a dashboard
pub async fn delete_dashboard_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<serde_json::Value> {
    // Check if dashboard exists
    if !state
        .dashboard_store
        .exists(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to check dashboard: {}", e)))?
    {
        return Err(ErrorResponse::not_found(format!(
            "Dashboard '{}' not found",
            id
        )));
    }

    state
        .dashboard_store
        .delete(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to delete dashboard: {}", e)))?;

    emit_dashboard_event(&state, &id, "delete");

    ok(serde_json::json!({
        "ok": true,
        "id": id,
    }))
}

/// Set default dashboard
pub async fn set_default_dashboard_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<serde_json::Value> {
    // Check if dashboard exists
    if !state
        .dashboard_store
        .exists(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to check dashboard: {}", e)))?
    {
        return Err(ErrorResponse::not_found(format!(
            "Dashboard '{}' not found",
            id
        )));
    }

    state
        .dashboard_store
        .set_default(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to set default dashboard: {}", e)))?;

    ok(serde_json::json!({
        "id": id,
        "is_default": true,
    }))
}

/// Request body for `PUT /api/dashboards/reorder`.
/// `dashboard_ids` is the desired full ordering (index 0 = top).
#[derive(Debug, Deserialize)]
pub struct ReorderDashboardsRequest {
    pub dashboard_ids: Vec<String>,
}

/// Response body for the reorder endpoint.
#[derive(Debug, Serialize)]
pub struct ReorderDashboardsResponse {
    pub ok: bool,
    pub count: usize,
}

/// Batch-update `sort_order` for a list of dashboards (PUT /api/dashboards/reorder)
///
/// Frontend sends the complete desired order as a list of dashboard IDs. We
/// assign each its index as the new `sort_order` and persist them in a single
/// transaction. Emits a single `DashboardUpdated` event with action `"reorder"`
/// so other clients refetch.
pub async fn reorder_dashboards_handler(
    State(state): State<ServerState>,
    Json(req): Json<ReorderDashboardsRequest>,
) -> HandlerResult<ReorderDashboardsResponse> {
    if req.dashboard_ids.is_empty() {
        return Err(ErrorResponse::bad_request(
            "dashboard_ids must not be empty",
        ));
    }

    let items: Vec<(String, i32)> = req
        .dashboard_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i as i32))
        .collect();

    let count = items.len();
    state
        .dashboard_store
        .set_sort_orders(&items)
        .map_err(|e| ErrorResponse::internal(format!("Failed to reorder dashboards: {}", e)))?;

    // Notify other clients. Use the first id as the event target — the SSE
    // consumer refetches the full list on receipt regardless of which id.
    if let Some(first_id) = req.dashboard_ids.first() {
        emit_dashboard_event(&state, first_id, "reorder");
    }

    ok(ReorderDashboardsResponse { ok: true, count })
}

/// List dashboard templates
pub async fn list_templates_handler(
    State(_state): State<ServerState>,
) -> HandlerResult<Vec<DashboardTemplate>> {
    let templates = default_templates();
    ok(templates.iter().map(stored_template_to_api).collect())
}

/// Get a template by ID
pub async fn get_template_handler(
    State(_state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<DashboardTemplate> {
    let templates = default_templates();
    let template = templates
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| ErrorResponse::not_found(format!("Template '{}' not found", id)))?;

    ok(stored_template_to_api(template))
}

// ============================================================================
// Share API Types
// ============================================================================

/// Share permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePermissions {
    pub allow_interactive: bool,
}

/// Request to create a share link
#[derive(Debug, Deserialize)]
pub struct CreateShareRequest {
    pub permissions: SharePermissions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_hours: Option<i64>,
}

/// Share token response
#[derive(Debug, Serialize)]
pub struct ShareTokenResponse {
    pub token: String,
    pub dashboard_id: String,
    pub permissions: SharePermissions,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub share_url: String,
}

/// Shared dashboard response (public)
#[derive(Debug, Serialize)]
pub struct SharedDashboardResponse {
    pub dashboard: Dashboard,
    pub permissions: SharePermissions,
    pub expires_at: Option<i64>,
}

// ============================================================================
// Share Handlers
// ============================================================================

/// Create a share link for a dashboard
pub async fn create_share_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(req): Json<CreateShareRequest>,
) -> HandlerResult<ShareTokenResponse> {
    // Verify dashboard exists
    state
        .dashboard_store
        .load(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load dashboard: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found(format!("Dashboard '{}' not found", id)))?;

    // Generate token: ds_ prefix + 22 random hex chars
    let random_bytes: [u8; 16] = rand::random();
    let token_str = format!("ds_{}", hex::encode(random_bytes));

    let now = chrono::Utc::now().timestamp();
    let expires_at = req.expires_in_hours.and_then(|h| {
        h.checked_mul(3600)
            .and_then(|seconds| now.checked_add(seconds))
    });

    let share = StoredShareToken {
        token: token_str.clone(),
        dashboard_id: id.clone(),
        permissions: StoredSharePermissions {
            allow_interactive: req.permissions.allow_interactive,
        },
        created_at: now,
        expires_at,
        created_by: None,
    };

    state
        .dashboard_store
        .save_share_token(&share)
        .map_err(|e| ErrorResponse::internal(format!("Failed to save share token: {}", e)))?;

    ok(ShareTokenResponse {
        token: token_str.clone(),
        dashboard_id: id.clone(),
        permissions: SharePermissions {
            allow_interactive: share.permissions.allow_interactive,
        },
        created_at: now,
        expires_at,
        share_url: format!("/share/{}", token_str),
    })
}

/// List all share links for a dashboard
pub async fn list_shares_handler(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> HandlerResult<Vec<ShareTokenResponse>> {
    let tokens = state
        .dashboard_store
        .list_share_tokens(&id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to list share tokens: {}", e)))?;

    let now = chrono::Utc::now().timestamp();
    let responses: Vec<ShareTokenResponse> = tokens
        .into_iter()
        .map(|t| ShareTokenResponse {
            share_url: format!("/share/{}", t.token),
            token: t.token,
            dashboard_id: t.dashboard_id,
            permissions: SharePermissions {
                allow_interactive: t.permissions.allow_interactive,
            },
            created_at: t.created_at,
            expires_at: t.expires_at,
        })
        .filter(|r| r.expires_at.is_none_or(|exp| exp > now))
        .collect();

    ok(responses)
}

/// Revoke a share link
pub async fn revoke_share_handler(
    State(state): State<ServerState>,
    Path((id, token)): Path<(String, String)>,
) -> HandlerResult<serde_json::Value> {
    // Verify the token belongs to this dashboard
    let share = state
        .dashboard_store
        .load_share_token(&token)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load share token: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found("Share link not found"))?;

    if share.dashboard_id != id {
        return Err(ErrorResponse::not_found("Share link not found"));
    }

    state
        .dashboard_store
        .delete_share_token(&token)
        .map_err(|e| ErrorResponse::internal(format!("Failed to delete share token: {}", e)))?;

    ok(serde_json::json!({
        "ok": true,
        "token": token,
    }))
}

/// Validate a share token: load it, check it exists
fn validate_share_token(
    state: &ServerState,
    token: &str,
) -> Result<StoredShareToken, ErrorResponse> {
    state
        .dashboard_store
        .load_share_token(token)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load share token: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found("Share link not found"))
}

/// Get shared dashboard data (public, no auth)
pub async fn get_shared_dashboard_handler(
    State(state): State<ServerState>,
    Path(token): Path<String>,
) -> HandlerResult<SharedDashboardResponse> {
    let share = validate_share_token(&state, &token)?;

    let dashboard = state
        .dashboard_store
        .load(&share.dashboard_id)
        .map_err(|e| ErrorResponse::internal(format!("Failed to load dashboard: {}", e)))?
        .ok_or_else(|| ErrorResponse::not_found("Dashboard not found"))?;

    // Check expiration
    if let Some(exp) = share.expires_at {
        if chrono::Utc::now().timestamp() > exp {
            return Err(ErrorResponse::new(
                "GONE",
                "This share link has expired",
                StatusCode::GONE,
            ));
        }
    }

    ok(SharedDashboardResponse {
        dashboard: stored_to_api(&dashboard),
        permissions: SharePermissions {
            allow_interactive: share.permissions.allow_interactive,
        },
        expires_at: share.expires_at,
    })
}

/// Proxy data requests through share token (public, no auth)
///
/// Forwards requests via localhost loopback to the same Axum server.
/// This avoids manually matching every API path — any GET/POST that
/// the existing router handles will work. We only enforce:
/// - Token validation + expiration check
/// - Read-only mode blocks write methods
/// - Sensitive admin/config paths are blocked
pub async fn share_proxy_handler(
    State(state): State<ServerState>,
    Path((token, path)): Path<(String, String)>,
    req: Request<Body>,
) -> Response {
    let method = req.method().clone();
    let headers = req.headers().clone();
    let query = req.uri().query().unwrap_or("").to_string();
    let body = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap_or_default();

    // 1. Validate share token
    let share = match validate_share_token(&state, &token) {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

    // 2. Check expiration
    if let Some(exp) = share.expires_at {
        if chrono::Utc::now().timestamp() > exp {
            return ErrorResponse::new("GONE", "This share link has expired", StatusCode::GONE)
                .into_response();
        }
    }

    let is_read_only = !share.permissions.allow_interactive;
    let path_str = path.as_ref();

    // 3. Enforce an ALLOWLIST of paths reachable through the share proxy.
    //    Previously this was a blocklist, which silently allowed anonymous
    //    share-token holders to read any device / telemetry / agent /
    //    extension data — far beyond what the shared dashboard needs. The
    //    semantics of a share link are "read the data this dashboard needs",
    //    not "read the entire platform".
    if !is_share_proxy_path_allowed(path_str) {
        return ErrorResponse::new(
            "FORBIDDEN",
            "This path is not accessible via share proxy",
            StatusCode::FORBIDDEN,
        )
        .into_response();
    }

    // 4. Block write methods in read-only mode
    if is_read_only && !is_allowed_readonly_method(path_str, &method) {
        return ErrorResponse::new(
            "FORBIDDEN",
            "This share link is read-only",
            StatusCode::FORBIDDEN,
        )
        .into_response();
    }

    // 5. Build query string
    let qs = if query.is_empty() {
        String::new()
    } else {
        format!("?{}", query)
    };
    let target_url = format!("http://127.0.0.1:9375/api/{}{}", path_str, qs);

    // 6. Forward via reqwest (internal loopback, skips auth middleware)
    let client = reqwest::Client::new();
    let mut req_builder = match method {
        Method::GET => client.get(&target_url),
        Method::POST => client.post(&target_url),
        Method::PUT => client.put(&target_url),
        Method::DELETE => client.delete(&target_url),
        _ => {
            return ErrorResponse::new(
                "METHOD_NOT_ALLOWED",
                "Method not supported",
                StatusCode::METHOD_NOT_ALLOWED,
            )
            .into_response();
        }
    };

    // Forward content-type header
    if let Some(ct) = headers.get("content-type") {
        req_builder = req_builder.header("content-type", ct);
    }

    // Mark as internal proxy so auth middleware bypasses JWT check.
    // MUST include the per-process secret — the auth middleware rejects
    // bypass attempts that lack it (blocks spoofed external requests).
    req_builder = req_builder.header("x-internal-proxy", "share").header(
        "x-internal-proxy-secret",
        state.internal_proxy_secret.as_str(),
    );

    if !body.is_empty() {
        req_builder = req_builder.body(body.to_vec());
    }

    match req_builder.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let ct = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();
            let body_bytes = resp.bytes().await.unwrap_or_default();

            Response::builder()
                .status(status)
                .header("content-type", ct)
                .body(Body::from(body_bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => ErrorResponse::internal(format!("Proxy request failed: {}", e)).into_response(),
    }
}

/// Paths reachable through the share proxy. This is an ALLOWLIST — anything
/// not explicitly listed is rejected by default.
///
/// A share link grants "read the data this dashboard needs" semantics. It
/// is NOT a general-purpose read-only account. Previously this used a
/// blocklist that silently allowed anonymous token holders to read every
/// device, telemetry series, agent execution, and extension output on the
/// platform.
///
/// Allowed categories:
/// - Telemetry queries (history + aggregates)
/// - Device list / detail / metrics (dashboard device widgets)
/// - Extension metric outputs (live data widgets)
/// - Agent execution history (status widgets)
/// - Data source values
/// - Messages (alerts shown on dashboard)
fn is_share_proxy_path_allowed(path: &str) -> bool {
    const ALLOWED_PREFIXES: &[&str] = &[
        "telemetry",
        "telemetry-stats",
        "devices",
        "device-types", // device type metadata for rendering device widgets; GET only, mutations blocked by method check
        "extensions", // list + metric/outputs subpaths; install/uninstall/write blocked by method check
        "agents", // list + execution history + detail; full CRUD blocked by method check in step 4
        "llm-backends", // model list for AI analyst widget; DTO exposes only api_key_configured, never the key
        "data-sources",
        "messages", // alerts shown on dashboard (list + detail); channel mutation blocked by method
        "frontend-components", // community widget manifests + bundles (GET only; install/uninstall blocked by method check)
    ];
    // First path segment (no query string).
    let first_segment = path.split('/').next().unwrap_or("");
    // Bare `messages` and `devices` (no trailing slash) also need to match.
    ALLOWED_PREFIXES
        .iter()
        .any(|p| path.starts_with(*p) || first_segment == *p)
}

#[cfg(test)]
mod share_proxy_tests {
    use super::is_share_proxy_path_allowed;

    #[test]
    fn allows_bare_list_endpoints() {
        // Regression: `extensions/` and `agents/` (trailing slash) blocked
        // the bare list GETs that shared-dashboard widgets call.
        for p in [
            "devices",
            "device-types",
            "extensions",
            "agents",
            "llm-backends",
            "messages",
            "frontend-components",
            "data-sources",
            "telemetry",
        ] {
            assert!(
                is_share_proxy_path_allowed(p),
                "bare `{p}` should be allowed"
            );
        }
    }

    #[test]
    fn allows_subpaths() {
        for p in [
            "devices/abc/current",
            "device-types/foo",
            "extensions/weather/command",
            "agents/xyz/executions",
            "telemetry/foo/values",
        ] {
            assert!(is_share_proxy_path_allowed(p), "`{p}` should be allowed");
        }
    }

    #[test]
    fn rejects_unlisted_paths() {
        for p in [
            "users",
            "api-keys",
            "settings",
            "instances",
            "rules",
            "dashboards",
        ] {
            assert!(!is_share_proxy_path_allowed(p), "`{p}` should be blocked");
        }
    }
}

/// Check if a path matches a pattern where `:id` matches any single segment.
/// e.g., `extensions/abc123/command` matches `extensions/:id/command`
fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    let path_segs: Vec<&str> = path.split('/').collect();
    let pat_segs: Vec<&str> = pattern.split('/').collect();
    if path_segs.len() != pat_segs.len() {
        return false;
    }
    path_segs
        .iter()
        .zip(pat_segs.iter())
        .all(|(p, pat)| pat.starts_with(':') || *p == *pat)
}

/// In read-only mode, only allow GET and specific safe POST endpoints.
/// Uses exact endpoint matching to prevent unauthorized access to side-effect
/// operations that share a prefix with read-like endpoints.
///
/// Share mode permission matrix:
/// ┌──────────────────────────────────────────┬────────┬─────────┐
/// │ Path pattern                             │ Method │ Allowed │
/// ├──────────────────────────────────────────┼────────┼─────────┤
/// │ * (unless blocked)                       │ GET    │ YES     │
/// │ extensions/:id/command                   │ POST   │ YES     │
/// │ devices/current-batch                    │ POST   │ YES     │
/// │ agents/:id/executions/details            │ POST   │ YES     │
/// │ * (all other)                            │ POST   │ NO      │
/// │ *                                        │ PUT    │ NO      │
/// │ *                                        │ DELETE │ NO      │
/// └──────────────────────────────────────────┴────────┴─────────┘
fn is_allowed_readonly_method(path: &str, method: &Method) -> bool {
    if method == Method::GET {
        return true;
    }
    if method == Method::POST {
        let allowed_post_patterns = [
            "extensions/:id/command",        // Extension read-only commands
            "devices/current-batch",         // Batch device current values
            "agents/:id/executions/details", // Batch get execution details
        ];
        return allowed_post_patterns
            .iter()
            .any(|p| path_matches_pattern(path, p));
    }
    false
}
