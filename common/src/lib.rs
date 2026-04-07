//! Shared types for the AI@home infrastructure.
//!
//! This crate defines the data models used by both the broker and node services.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Describes an AI capability offered by a node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    /// Short identifier, e.g. "text-generation", "image-classification".
    pub name: String,
    /// Optional human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Information about a registered AI node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique node identifier assigned by the broker on registration.
    pub id: Uuid,
    /// Human-readable name for this node.
    pub name: String,
    /// Base URL at which the node's HTTP API is reachable.
    pub url: String,
    /// Capabilities advertised by this node.
    pub capabilities: Vec<Capability>,
    /// When the node registered with the broker.
    pub registered_at: DateTime<Utc>,
}

// ── Broker API request / response bodies ────────────────────────────────────

/// Request body sent by a node when registering with the broker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Human-readable name for this node.
    pub name: String,
    /// Base URL at which the node's HTTP API is reachable.
    pub url: String,
    /// Capabilities advertised by this node.
    pub capabilities: Vec<Capability>,
}

/// Response body returned by the broker after a successful registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    /// The node identifier assigned by the broker.
    pub id: Uuid,
    /// Echo of the registered URL.
    pub url: String,
}

// ── Node API request / response bodies ───────────────────────────────────────

/// Request body sent to a node for inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferRequest {
    /// The capability to invoke (must match one the node advertised).
    pub capability: String,
    /// Arbitrary JSON payload consumed by the capability handler.
    pub input: serde_json::Value,
}

/// Response body returned by a node after processing an inference request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferResponse {
    /// The capability that handled the request.
    pub capability: String,
    /// Arbitrary JSON payload produced by the capability handler.
    pub output: serde_json::Value,
}

// ── Health check ─────────────────────────────────────────────────────────────

/// Standard health-check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
        }
    }
}
