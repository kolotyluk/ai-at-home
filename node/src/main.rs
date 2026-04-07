//! Node server for AI@home.
//!
//! Each node:
//!   1. Starts an HTTP API that handles inference requests.
//!   2. Registers itself with the broker on startup.
//!   3. Deregisters from the broker on graceful shutdown.
//!
//! Environment variables
//! ---------------------
//! `NODE_NAME`        – human-readable name (default: "ai-node")
//! `NODE_PORT`        – port to bind the node's HTTP API (default: 4000)
//! `NODE_PUBLIC_URL`  – URL advertised to the broker (default: "http://localhost:<NODE_PORT>")
//! `BROKER_URL`       – base URL of the broker (default: "http://localhost:3000")

use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use common::{Capability, HealthResponse, InferRequest, InferResponse, RegisterRequest};
use reqwest::Client;
use tracing::{error, info, warn};
use uuid::Uuid;

// ── Node state ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NodeState {
    /// Human-readable name for this node.
    pub name: String,
    /// Capabilities advertised to the broker.
    pub capabilities: Vec<Capability>,
    /// Base URL of the broker service.
    pub broker_url: String,
    /// Node id assigned by the broker (None until registered).
    pub node_id: Arc<RwLock<Option<Uuid>>>,
    /// HTTP client used to communicate with the broker.
    pub http_client: Client,
}

// ── Application ───────────────────────────────────────────────────────────────

/// Build the Axum router (exported so tests can reuse it without binding a port).
pub fn build_router(state: NodeState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/infer", post(infer_handler))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "node=info,tower_http=debug".into()),
        )
        .init();

    let name = std::env::var("NODE_NAME").unwrap_or_else(|_| "ai-node".to_string());
    let port: u16 = std::env::var("NODE_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4000);
    let public_url = std::env::var("NODE_PUBLIC_URL")
        .unwrap_or_else(|_| format!("http://localhost:{port}"));
    let broker_url =
        std::env::var("BROKER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let capabilities = vec![
        Capability {
            name: "text-generation".to_string(),
            description: Some("Simple echo-based text generation".to_string()),
        },
        Capability {
            name: "echo".to_string(),
            description: Some("Echoes the input back as output".to_string()),
        },
    ];

    let state = NodeState {
        name: name.clone(),
        capabilities: capabilities.clone(),
        broker_url: broker_url.clone(),
        node_id: Arc::new(RwLock::new(None)),
        http_client: Client::new(),
    };

    // Register with the broker
    let node_id_handle = Arc::clone(&state.node_id);
    register_with_broker(&state.http_client, &broker_url, &name, &public_url, &capabilities, &node_id_handle).await;

    let app = build_router(state.clone());

    let addr: SocketAddr = format!("0.0.0.0:{port}").parse().unwrap();
    info!("Node '{name}' listening on {addr}");

    // Serve until a shutdown signal is received, then deregister
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let serve_future = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());
    serve_future.await.unwrap();

    // Deregister from broker on clean shutdown
    deregister_from_broker(&state.http_client, &broker_url, &node_id_handle).await;
    info!("Node '{name}' shut down cleanly");
}

/// Register this node with the broker.
async fn register_with_broker(
    client: &Client,
    broker_url: &str,
    name: &str,
    public_url: &str,
    capabilities: &[Capability],
    node_id_handle: &Arc<RwLock<Option<Uuid>>>,
) {
    let req = RegisterRequest {
        name: name.to_string(),
        url: public_url.to_string(),
        capabilities: capabilities.to_vec(),
    };

    match client
        .post(format!("{broker_url}/api/v1/nodes"))
        .json(&req)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<common::RegisterResponse>().await {
                Ok(body) => {
                    *node_id_handle.write().unwrap() = Some(body.id);
                    info!("Registered with broker – node id = {}", body.id);
                }
                Err(e) => error!("Failed to parse broker registration response: {e}"),
            }
        }
        Ok(resp) => warn!("Broker returned error on registration: {}", resp.status()),
        Err(e) => warn!("Could not reach broker for registration: {e}"),
    }
}

/// Deregister this node from the broker.
async fn deregister_from_broker(
    client: &Client,
    broker_url: &str,
    node_id_handle: &Arc<RwLock<Option<Uuid>>>,
) {
    let id = *node_id_handle.read().unwrap();
    if let Some(id) = id {
        match client
            .delete(format!("{broker_url}/api/v1/nodes/{id}"))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() || resp.status() == 204 => {
                info!("Deregistered from broker (id={id})");
            }
            Ok(resp) => warn!("Broker returned error on deregistration: {}", resp.status()),
            Err(e) => warn!("Could not reach broker for deregistration: {e}"),
        }
    }
}

/// Wait for SIGINT / SIGTERM.
async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /health`
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse::ok())
}

/// `POST /api/v1/infer`
///
/// Runs inference for the given capability.  Currently supported capabilities:
/// - `text-generation`: echoes the `prompt` field wrapped in a response.
/// - `echo`:            echoes the entire input payload back.
async fn infer_handler(
    State(state): State<NodeState>,
    Json(req): Json<InferRequest>,
) -> impl IntoResponse {
    let capability_supported = state
        .capabilities
        .iter()
        .any(|c| c.name == req.capability);

    if !capability_supported {
        let msg = format!("Capability '{}' not supported by this node", req.capability);
        warn!("{msg}");
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response();
    }

    let output = match req.capability.as_str() {
        "text-generation" => {
            let prompt = req
                .input
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            serde_json::json!({
                "text": format!("[generated] {prompt}")
            })
        }
        "echo" | _ => req.input.clone(),
    };

    info!("Handled '{}' request", req.capability);
    Json(InferResponse {
        capability: req.capability,
        output,
    })
    .into_response()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;

    fn make_server() -> TestServer {
        let state = NodeState {
            name: "test-node".to_string(),
            capabilities: vec![
                Capability {
                    name: "text-generation".to_string(),
                    description: None,
                },
                Capability {
                    name: "echo".to_string(),
                    description: None,
                },
            ],
            broker_url: "http://localhost:3000".to_string(),
            node_id: Arc::new(RwLock::new(None)),
            http_client: Client::new(),
        };
        TestServer::new(build_router(state)).unwrap()
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let server = make_server();
        let resp = server.get("/health").await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn text_generation_returns_generated_text() {
        let server = make_server();
        let resp = server
            .post("/api/v1/infer")
            .json(&json!({
                "capability": "text-generation",
                "input": {"prompt": "Hello, world!"}
            }))
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["capability"], "text-generation");
        let text = body["output"]["text"].as_str().unwrap();
        assert!(text.contains("Hello, world!"), "unexpected output: {text}");
    }

    #[tokio::test]
    async fn echo_returns_input() {
        let server = make_server();
        let payload = json!({"foo": "bar", "num": 42});
        let resp = server
            .post("/api/v1/infer")
            .json(&json!({
                "capability": "echo",
                "input": payload
            }))
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["output"], payload);
    }

    #[tokio::test]
    async fn unsupported_capability_returns_422() {
        let server = make_server();
        let resp = server
            .post("/api/v1/infer")
            .json(&json!({
                "capability": "unknown-capability",
                "input": {}
            }))
            .await;
        resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn text_generation_empty_prompt() {
        let server = make_server();
        let resp = server
            .post("/api/v1/infer")
            .json(&json!({
                "capability": "text-generation",
                "input": {}
            }))
            .await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        // Even with empty prompt the handler should succeed
        assert_eq!(body["capability"], "text-generation");
    }
}
