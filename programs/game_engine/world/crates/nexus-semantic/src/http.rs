//! Bone 3b — HTTP entry point.
//!
//! Two routes. One purpose: make the semantic network addressable from outside.
//!
//!   POST /packet           — enqueue a text packet, return chain_id immediately
//!   GET  /chain/{chain_id} — retrieve all terminal records for a chain
//!
//! Request body (POST /packet):
//!   { "text": "...", "origin": "dworld://..." }
//!   `origin` is optional — defaults to "dworld://http/".
//!   `depth_limit` is optional u32 — defaults to 16.
//!
//! Response (POST /packet):
//!   { "chain_id": 42, "packet_id": 1 }
//!
//! Response (GET /chain/{id}):
//!   { "records": [ { "chain_id": 42, "packet_id": 1, "hop_count": 3,
//!                    "identity": "...", "output": "...",
//!                    "world_position": [x,y,z] | null,
//!                    "quality": 0.9 | null } ] }
//!
//! The caller polls GET /chain/{id} until records appear.
//! Bone 3d (SSE or WebSocket streaming) can replace polling later.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};

use nexus_core::types::{PacketData, SemanticPacket};
use nexus_events::EventLog;

use crate::worker::RoutingLoop;

// ─── Shared state ─────────────────────────────────────────────────────────────

/// Everything the HTTP handlers need.
pub struct HttpState {
    pub routing_loop: Arc<RoutingLoop>,
    pub event_log: Arc<EventLog>,
    /// Monotonic chain ID counter. Each POST /packet increments this.
    pub next_chain_id: Arc<AtomicU64>,
}

// ─── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PacketRequest {
    pub text: String,
    #[serde(default = "default_origin")]
    pub origin: String,
    /// Maximum routing depth. Defaults to 16.
    pub depth_limit: Option<u32>,
}

fn default_origin() -> String {
    "dworld://http/".to_string()
}

#[derive(Serialize)]
pub struct PacketResponse {
    pub chain_id: u64,
    pub packet_id: u64,
}

#[derive(Serialize)]
pub struct ChainRecord {
    pub chain_id: u64,
    pub packet_id: u64,
    pub hop_count: u32,
    pub identity: String,
    pub output: String,
    pub world_position: Option<[f32; 3]>,
    pub quality: Option<f32>,
}

#[derive(Serialize)]
pub struct ChainResponse {
    pub records: Vec<ChainRecord>,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// POST /packet
///
/// Assigns a chain_id, creates a SemanticPacket, submits it to the routing
/// loop, and returns immediately. The caller polls GET /chain/{id}.
async fn post_packet(
    State(state): State<Arc<HttpState>>,
    Json(req): Json<PacketRequest>,
) -> impl IntoResponse {
    let chain_id = state.next_chain_id.fetch_add(1, Ordering::Relaxed);
    let packet_id = 1u64; // first packet in the chain

    let mut packet = SemanticPacket::new(
        packet_id,
        chain_id,
        PacketData::Text(req.text),
        req.origin,
    );

    if let Some(limit) = req.depth_limit {
        packet.depth_limit = limit;
    }

    state.routing_loop.submit(packet).await;

    Json(PacketResponse { chain_id, packet_id }).into_response()
}

/// GET /chain/{chain_id}
///
/// Returns all EventLog records for the given chain.
/// Returns an empty records list if routing is still in progress.
async fn get_chain(
    State(state): State<Arc<HttpState>>,
    Path(chain_id): Path<u64>,
) -> impl IntoResponse {
    match state.event_log.chain(chain_id) {
        Ok(records) => {
            let out: Vec<ChainRecord> = records
                .into_iter()
                .map(|r| ChainRecord {
                    chain_id: r.chain_id,
                    packet_id: r.packet_id,
                    hop_count: r.hop_count,
                    identity: r.identity,
                    output: r.output,
                    world_position: r.world_position,
                    quality: r.quality,
                })
                .collect();
            Json(ChainResponse { records: out }).into_response()
        }
        Err(e) => {
            tracing::error!("event log query failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /health
///
/// Returns 200 OK. Load balancers and Docker health checks use this.
async fn health() -> impl IntoResponse {
    StatusCode::OK
}

// ─── Router ───────────────────────────────────────────────────────────────────

/// Build the axum Router for the semantic HTTP API.
///
/// Bind with: `axum::serve(listener, router(state)).await`
pub fn router(state: Arc<HttpState>) -> Router {
    Router::new()
        .route("/packet", post(post_packet))
        .route("/chain/{chain_id}", get(get_chain))
        .route("/health", get(health))
        .with_state(state)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{self, Request};
    use axum::body::Body;
    use tower::ServiceExt; // for `oneshot`
    use crate::identity::{seed_identities, IdentityStore};
    use crate::llm::MockLlmClient;

    fn make_state() -> Arc<HttpState> {
        let store = IdentityStore::build(seed_identities());
        let llm = Arc::new(MockLlmClient::new());
        let log = Arc::new(EventLog::open_in_memory().unwrap());
        let (routing_loop, rx) = RoutingLoop::new(store, llm, Arc::clone(&log));
        let _handle = Arc::clone(&routing_loop).spawn(rx);
        Arc::new(HttpState {
            routing_loop,
            event_log: log,
            next_chain_id: Arc::new(AtomicU64::new(1)),
        })
    }

    #[tokio::test]
    async fn post_packet_returns_chain_id() {
        let state = make_state();
        let app = router(state);

        let body = r#"{"text":"what is the semantic field?"}"#;
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/packet")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["chain_id"], 1);
        assert_eq!(json["packet_id"], 1);
    }

    #[tokio::test]
    async fn get_chain_returns_empty_before_terminal() {
        let state = make_state();
        let app = router(state);

        let req = Request::builder()
            .method(http::Method::GET)
            .uri("/chain/999")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["records"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn health_returns_200() {
        let state = make_state();
        let app = router(state);

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_then_poll_chain_finds_record() {
        let state = make_state();
        let app = router(Arc::clone(&state));

        // Submit a packet with depth_limit=1 so it terminates in one hop
        let body = r#"{"text":"describe the routing loop","depth_limit":1}"#;
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/packet")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let chain_id = json["chain_id"].as_u64().unwrap();

        // Wait for routing to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Poll the chain
        let req2 = Request::builder()
            .uri(format!("/chain/{chain_id}"))
            .body(Body::empty())
            .unwrap();

        let resp2 = app.oneshot(req2).await.unwrap();
        let bytes2 = axum::body::to_bytes(resp2.into_body(), 4096).await.unwrap();
        let json2: serde_json::Value = serde_json::from_slice(&bytes2).unwrap();

        assert_eq!(
            json2["records"].as_array().unwrap().len(),
            1,
            "one terminal record expected for chain {chain_id}"
        );
    }
}
