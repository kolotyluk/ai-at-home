//! Broker service for AI@home.
//!
//! The broker is the central directory that:
//!   - Accepts node registrations  (`POST   /api/v1/nodes`)
//!   - Accepts node deregistrations (`DELETE /api/v1/nodes/:id`)
//!   - Returns the list of live nodes to clients (`GET /api/v1/nodes`)
//!   - Answers health-check pings (`GET /health`)

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use common::{HealthResponse, NodeInfo, RegisterRequest, RegisterResponse};
use tracing::info;
use uuid::Uuid;

// ── Shared state ─────────────────────────────────────────────────────────────

type NodeRegistry = Arc<RwLock<HashMap<Uuid, NodeInfo>>>;

// ── Application ───────────────────────────────────────────────────────────────

/// Build the Axum router (exported so tests can reuse it without binding a port).
pub fn build_router(registry: NodeRegistry) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/nodes", post(register_handler))
        .route("/api/v1/nodes", get(list_nodes_handler))
        .route("/api/v1/nodes/:id", delete(deregister_handler))
        .with_state(registry)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "broker=info,tower_http=debug".into()),
        )
        .init();

    let registry: NodeRegistry = Arc::new(RwLock::new(HashMap::new()));
    let app = build_router(registry);

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    info!("Broker listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /health`
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse::ok())
}

/// `POST /api/v1/nodes`
///
/// A node calls this endpoint to register itself with the broker.
async fn register_handler(
    State(registry): State<NodeRegistry>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let node = NodeInfo {
        id,
        name: req.name.clone(),
        url: req.url.clone(),
        capabilities: req.capabilities,
        registered_at: Utc::now(),
    };

    registry.write().unwrap().insert(id, node);
    info!("Registered node '{}' (id={id}) at {}", req.name, req.url);

    (
        StatusCode::CREATED,
        Json(RegisterResponse {
            id,
            url: req.url,
        }),
    )
}

/// `GET /api/v1/nodes`
///
/// Returns all currently registered nodes.
async fn list_nodes_handler(State(registry): State<NodeRegistry>) -> Json<Vec<NodeInfo>> {
    let nodes: Vec<NodeInfo> = registry.read().unwrap().values().cloned().collect();
    Json(nodes)
}

/// `DELETE /api/v1/nodes/:id`
///
/// A node calls this endpoint to deregister itself from the broker.
async fn deregister_handler(
    State(registry): State<NodeRegistry>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match registry.write().unwrap().remove(&id) {
        Some(node) => {
            info!("Deregistered node '{}' (id={id})", node.name);
            StatusCode::NO_CONTENT
        }
        None => StatusCode::NOT_FOUND,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;

    fn make_server() -> TestServer {
        let registry: NodeRegistry = Arc::new(RwLock::new(HashMap::new()));
        TestServer::new(build_router(registry)).unwrap()
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
    async fn register_and_list_node() {
        let server = make_server();

        // Register a node
        let resp = server
            .post("/api/v1/nodes")
            .json(&json!({
                "name": "test-node",
                "url": "http://localhost:4000",
                "capabilities": [{"name": "text-generation"}]
            }))
            .await;
        resp.assert_status(StatusCode::CREATED);
        let reg: serde_json::Value = resp.json();
        let id = reg["id"].as_str().unwrap().to_string();

        // List nodes – should contain the one we just registered
        let resp = server.get("/api/v1/nodes").await;
        resp.assert_status_ok();
        let nodes: Vec<serde_json::Value> = resp.json();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["id"], id);
        assert_eq!(nodes[0]["name"], "test-node");
    }

    #[tokio::test]
    async fn deregister_node() {
        let server = make_server();

        // Register
        let resp = server
            .post("/api/v1/nodes")
            .json(&json!({
                "name": "node-to-remove",
                "url": "http://localhost:4001",
                "capabilities": []
            }))
            .await;
        let reg: serde_json::Value = resp.json();
        let id = reg["id"].as_str().unwrap().to_string();

        // Deregister
        let resp = server.delete(&format!("/api/v1/nodes/{id}")).await;
        resp.assert_status(StatusCode::NO_CONTENT);

        // List – should now be empty
        let resp = server.get("/api/v1/nodes").await;
        let nodes: Vec<serde_json::Value> = resp.json();
        assert!(nodes.is_empty());
    }

    #[tokio::test]
    async fn deregister_unknown_node_returns_404() {
        let server = make_server();
        let fake_id = Uuid::new_v4();
        let resp = server
            .delete(&format!("/api/v1/nodes/{fake_id}"))
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn multiple_nodes_can_register() {
        let server = make_server();

        for i in 0..3 {
            server
                .post("/api/v1/nodes")
                .json(&json!({
                    "name": format!("node-{i}"),
                    "url": format!("http://localhost:{}", 5000 + i),
                    "capabilities": [{"name": "text-generation", "description": "LLM inference"}]
                }))
                .await
                .assert_status(StatusCode::CREATED);
        }

        let resp = server.get("/api/v1/nodes").await;
        let nodes: Vec<serde_json::Value> = resp.json();
        assert_eq!(nodes.len(), 3);
    }

    #[tokio::test]
    async fn register_response_contains_url() {
        let server = make_server();
        let url = "http://my-node.local:4000";
        let resp = server
            .post("/api/v1/nodes")
            .json(&json!({
                "name": "url-check-node",
                "url": url,
                "capabilities": []
            }))
            .await;
        let body: serde_json::Value = resp.json();
        assert_eq!(body["url"], url);
    }

    #[tokio::test]
    async fn capabilities_are_stored() {
        let server = make_server();
        let resp = server
            .post("/api/v1/nodes")
            .json(&json!({
                "name": "capable-node",
                "url": "http://localhost:6000",
                "capabilities": [
                    {"name": "text-generation", "description": "LLM"},
                    {"name": "image-classification"}
                ]
            }))
            .await;
        let reg: serde_json::Value = resp.json();
        let id = reg["id"].as_str().unwrap();

        let nodes: Vec<serde_json::Value> = server.get("/api/v1/nodes").await.json();
        let node = nodes.iter().find(|n| n["id"] == id).unwrap();
        assert_eq!(node["capabilities"].as_array().unwrap().len(), 2);
    }
}
