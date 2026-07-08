//! Server middleware.

use axum::{
    body::Body, extract::ConnectInfo, extract::State, http::Request, middleware::Next,
    response::IntoResponse,
};
use std::net::SocketAddr;

use super::types::ServerState;
use crate::rate_limit::extract_client_id;

/// Rate limiting middleware.
///
/// Uses API key (if authenticated) or IP address for rate limiting.
/// Public endpoints have higher limits; protected endpoints have standard limits.
/// WebSocket and SSE endpoints are excluded from rate limiting.
///
pub async fn rate_limit_middleware(
    State(state): State<ServerState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let uri = request.uri().path();
    if uri.contains("/chat") || uri.contains("/ws") || uri.contains("/events/stream") {
        return next.run(request).await;
    }

    let client_id = extract_client_id(request.headers(), connect_info.as_ref());

    match state.rate_limiter.check_rate_limit(&client_id) {
        Ok(_) => next.run(request).await,
        Err(e) => {
            if e.should_log() {
                tracing::warn!(
                    category = "rate_limit",
                    client = %client_id,
                    wait_seconds = e.wait_seconds,
                    "Rate limit exceeded"
                );
            }
            e.into_response()
        }
    }
}

/// Webhook-specific rate-limit middleware.
///
/// Same engine as `rate_limit_middleware`, but produces a composite `client_id`
/// that embeds the `device_id` from the URL path. Without this, devices that
/// share an adapter-level `X-API-Key` would all land in the same rate-limit
/// bucket (`apikey:<hash>`), so one chatty device could starve its neighbors.
///
/// Per-device endpoint (`POST /api/devices/:id/webhook`) → bucket becomes
/// `apikey:<hash>:<device_id>` (or `ip:<addr>:<device_id>` when no API key is
/// present), giving each device its own quota even under a shared key.
///
/// Generic endpoint (`POST /api/devices/webhook`) — the device_id lives in the
/// request body and can't be read here without consuming it. Devices on this
/// endpoint that share an adapter API key share a rate-limit bucket. Workaround:
/// prefer the per-device URL endpoint, or configure per-device `webhook_token`s
/// (each gets a unique `Authorization: Bearer` hash → unique bucket).
pub async fn webhook_rate_limit_middleware(
    State(state): State<ServerState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let uri = request.uri().path();
    let headers = request.headers();

    let base_id = extract_client_id(headers, connect_info.as_ref());

    // Extract device_id from /api/devices/:id/webhook. The literal `webhook`
    // segment is filtered out so the generic endpoint (`/api/devices/webhook`)
    // falls through to the un-composited base_id.
    let device_segment = uri
        .strip_prefix("/api/devices/")
        .and_then(|rest| rest.split('/').next())
        .filter(|seg| !seg.is_empty() && *seg != "webhook");

    let client_id = match device_segment {
        Some(id) => format!("{}:{}", base_id, id),
        None => base_id,
    };

    match state.rate_limiter.check_rate_limit(&client_id) {
        Ok(_) => next.run(request).await,
        Err(e) => {
            if e.should_log() {
                tracing::warn!(
                    category = "rate_limit",
                    client = %client_id,
                    wait_seconds = e.wait_seconds,
                    "Webhook rate limit exceeded"
                );
            }
            e.into_response()
        }
    }
}
