//! Model discovery for named cloud providers. Keys stay in POST bodies and
//! are forwarded only to the provider's fixed HTTPS endpoint, without redirects.

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

use super::common::{ok, HandlerResult};
use crate::models::ErrorResponse;

const AGENTROUTER_MODELS_URL: &str = "https://agentrouter.org/v1/models";
const MAX_CATALOG_BYTES: usize = 1024 * 1024;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AgentRouterModelsRequest {
    pub api_key: Option<String>,
    /// Reuse a stored key only when its backend belongs to AgentRouter.
    pub backend_id: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CloudModel {
    pub id: String,
    pub name: String,
}

fn is_agentrouter_endpoint(endpoint: &str) -> bool {
    reqwest::Url::parse(endpoint).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str() == Some("agentrouter.org")
            && url.port_or_known_default() == Some(443)
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && matches!(url.path().trim_end_matches('/'), "" | "/v1")
    })
}

fn resolve_key(request: AgentRouterModelsRequest) -> Result<String, ErrorResponse> {
    if let Some(key) = request.api_key.filter(|key| !key.trim().is_empty()) {
        return Ok(key.trim().to_string());
    }
    let id = request
        .backend_id
        .ok_or_else(|| ErrorResponse::bad_request("Enter an AgentRouter API key first"))?;
    let manager = heramind_agent::llm_backends::get_instance_manager()
        .map_err(|_| ErrorResponse::service_unavailable("LLM backend manager is unavailable"))?;
    let instance = manager
        .get_instance(&id)
        .ok_or_else(|| ErrorResponse::not_found("LLM backend"))?;
    if !is_agentrouter_endpoint(instance.endpoint.as_deref().unwrap_or_default()) {
        return Err(ErrorResponse::bad_request(
            "The stored API key does not belong to an AgentRouter backend",
        ));
    }
    instance
        .api_key
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| ErrorResponse::bad_request("No AgentRouter API key is configured"))
}

fn parse_models(body: &[u8]) -> Result<Vec<CloudModel>, ErrorResponse> {
    let value: serde_json::Value = serde_json::from_slice(body).map_err(|_| {
        ErrorResponse::service_unavailable("AgentRouter returned an invalid model catalog")
    })?;
    let data = value["data"].as_array().ok_or_else(|| {
        ErrorResponse::service_unavailable("AgentRouter returned an unsupported model catalog")
    })?;
    let mut models = BTreeMap::new();
    for model in data {
        let Some(id) = model["id"].as_str().map(str::trim) else {
            continue;
        };
        if id.is_empty() || id.len() > 512 || id.chars().any(char::is_control) {
            continue;
        }
        let name = model["name"]
            .as_str()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or(id);
        models.entry(id.to_string()).or_insert_with(|| CloudModel {
            id: id.to_string(),
            name: name.to_string(),
        });
    }
    Ok(models.into_values().collect())
}

async fn fetch_models(url: &str, key: &str) -> Result<Vec<CloudModel>, ErrorResponse> {
    if key.is_empty()
        || key.len() > 4096
        || !key.is_ascii()
        || key.bytes().any(|b| b.is_ascii_control())
    {
        return Err(ErrorResponse::bad_request("Invalid AgentRouter API key"));
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| ErrorResponse::internal("Could not create the provider client"))?;
    let mut response = client.get(url).bearer_auth(key).send().await.map_err(|_| {
        ErrorResponse::service_unavailable("Could not reach AgentRouter; please try again")
    })?;
    if !response.status().is_success() {
        // Never echo upstream bodies: providers can include credentials in errors.
        let message = match response.status().as_u16() {
            401 | 403 => "AgentRouter rejected access. Check the API key and whether this client is allowed to access the model catalog.",
            429 => "AgentRouter rate limit reached. Please try again later.",
            404 | 405 => "AgentRouter does not expose a model catalog for this account. Enter the model ID manually.",
            _ => "AgentRouter could not return the model catalog. Please try again later.",
        };
        return Err(ErrorResponse::new(
            "PROVIDER_MODELS_ERROR",
            message,
            StatusCode::BAD_GATEWAY,
        ));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| {
        ErrorResponse::service_unavailable("Could not read the AgentRouter model catalog")
    })? {
        if body.len() + chunk.len() > MAX_CATALOG_BYTES {
            return Err(ErrorResponse::service_unavailable(
                "AgentRouter model catalog is too large",
            ));
        }
        body.extend_from_slice(&chunk);
    }
    parse_models(&body)
}

/// Discover account-visible model IDs without saving a backend or API key.
#[utoipa::path(
    post,
    path = "/api/llm-backends/providers/agentrouter/models",
    tag = "llm-backends",
    request_body = AgentRouterModelsRequest,
    responses((status = 200, description = "AgentRouter model catalog"), (status = 502, description = "Provider rejected discovery"))
)]
pub async fn list_agentrouter_models_handler(
    Json(request): Json<AgentRouterModelsRequest>,
) -> HandlerResult<serde_json::Value> {
    let key = resolve_key(request)?;
    let models = fetch_models(AGENTROUTER_MODELS_URL, &key).await?;
    ok(serde_json::json!({ "models": models }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};

    #[test]
    fn catalog_preserves_provider_ids_and_ignores_invalid_entries() {
        let models = parse_models(br#"{"data":[{"id":"glm-5.3"},{"id":"claude-opus-4-8","name":"Claude"},{"id":"glm-5.3"},{"id":""},{"id":42},{}]}"#).unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "claude-opus-4-8");
        assert_eq!(models[0].name, "Claude");
        assert_eq!(models[1].name, "glm-5.3");
        assert!(parse_models(br#"{"error":"invalid key"}"#).is_err());
        assert!(parse_models(br#"<html>challenge</html>"#).is_err());
        assert!(parse_models(br#"{"data":[]}"#).unwrap().is_empty());
    }

    #[test]
    fn saved_keys_only_belong_to_exact_provider_origin() {
        for endpoint in ["https://agentrouter.org", "https://agentrouter.org/v1/"] {
            assert!(is_agentrouter_endpoint(endpoint));
        }
        for endpoint in [
            "http://agentrouter.org/v1",
            "https://agentrouter.org.evil.com/v1",
            "https://evil.com",
            "https://agentrouter.org:444/v1",
            "https://user@agentrouter.org/v1",
            "https://agentrouter.org/v1?next=evil",
            "https://agentrouter.org/other",
        ] {
            assert!(!is_agentrouter_endpoint(endpoint));
        }
    }

    #[tokio::test]
    async fn discovery_authenticates_and_does_not_follow_redirects_or_echo_secrets() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let router = Router::new()
            .route(
                "/models",
                get(|headers: axum::http::HeaderMap| async move {
                    assert_eq!(headers["authorization"], "Bearer test-only-key");
                    Json(serde_json::json!({"data": [{"id": "account-model"}]}))
                }),
            )
            .route(
                "/redirect",
                get(|| async { axum::response::Redirect::temporary("/models") }),
            )
            .route(
                "/denied",
                get(|| async { (StatusCode::UNAUTHORIZED, "test-only-key") }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let models = fetch_models(&format!("{base}/models"), "test-only-key")
            .await
            .unwrap();
        assert_eq!(models[0].id, "account-model");
        assert!(fetch_models(&format!("{base}/redirect"), "test-only-key")
            .await
            .is_err());
        let error = fetch_models(&format!("{base}/denied"), "test-only-key")
            .await
            .unwrap_err();
        assert!(!error.message.contains("test-only-key"));
        server.abort();
    }
}
