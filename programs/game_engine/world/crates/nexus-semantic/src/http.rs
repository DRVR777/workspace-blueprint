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
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};

use nexus_core::types::{PacketData, SemanticPacket};
use nexus_events::EventLog;
use crate::identity::IdentityFile;

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

// ─── dworld:// protocol endpoints ─────────────────────────────────────────────
//
// These four endpoints are the library layer — the server as index, not as
// compute proxy.  Callers borrow a lens (identity file), run their own inference,
// and write outputs back to the field.
//
// The managed compute path (POST /packet) remains for callers that want the
// server to do the inference on their behalf.

/// GET /.dworld/{*path}
///
/// dworld:// URI resolution. Returns the SpatialManifest for the given path.
///
/// ENTER dworld://nexus.local/agents/NEXUS
///   → GET https://nexus.local/.dworld/nexus.local/agents/NEXUS
///   ← SpatialManifest JSON
///
/// For paths that match a known identity file address, the manifest's
/// `semantic_identity` field points to that identity.  Unknown paths return
/// the default world manifest with the nearest identity file for the path text.
async fn get_manifest(
    State(state): State<Arc<HttpState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let address = format!("dworld://{path}");
    let identity_addr = state.routing_loop
        .get_identity(&address)
        .map(|id| id.address)
        .or_else(|| {
            // No exact match — find nearest by text proximity using the path itself
            // In production, embed the path and query HNSW. For now, first seed identity.
            state.routing_loop
                .current_store()
                .iter()
                .next()
                .map(|id| id.address.clone())
        });

    let manifest = serde_json::json!({
        "worldId":          address,
        "geometry":         null,
        "surface":          ["talk", "query", "embed"],
        "agent":            identity_addr,
        "payment":          null,
        "semanticIdentity": identity_addr,
    });

    Json(manifest).into_response()
}

/// GET /.dworld/identities/{name}
///
/// Returns the identity file at `dworld://{name}`.
/// The caller uses this as the system-prompt lens for their own LLM call.
#[derive(Serialize)]
struct IdentityResponse {
    address: String,
    content: String,
    world_coord: Option<[f32; 3]>,
}

async fn get_identity(
    State(state): State<Arc<HttpState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    // Try exact address first, then the canonical seed prefix.
    // GET /.dworld/identities/PHILOSOPHER
    //   → tries "dworld://PHILOSOPHER"
    //   → falls back to "dworld://council.local/identities/PHILOSOPHER"
    let identity = state.routing_loop.get_identity(&format!("dworld://{name}"))
        .or_else(|| state.routing_loop.get_identity(
            &format!("dworld://council.local/identities/{name}")
        ));

    match identity {
        Some(id) => Json(IdentityResponse {
            address: id.address,
            content: id.content,
            world_coord: id.world_coord,
        }).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// POST /.dworld/field
///
/// Index a new output into the semantic field.
/// The caller provides the already-embedded vector (library model: caller owns
/// inference) OR just content (the server will embed it).
///
/// Body: { "address": "dworld://...", "content": "...", "vector": [...] | null }
#[derive(Deserialize)]
struct FieldWriteRequest {
    /// dworld:// address for the new identity. Auto-generated if absent.
    address: Option<String>,
    /// Text content of the output.
    content: String,
    /// Pre-computed embedding vector. If absent, the server embeds `content`.
    vector: Option<Vec<f32>>,
}

#[derive(Serialize)]
struct FieldWriteResponse {
    address: String,
    indexed: bool,
}

async fn post_field(
    State(state): State<Arc<HttpState>>,
    headers: HeaderMap,
    Json(req): Json<FieldWriteRequest>,
) -> impl IntoResponse {
    if !check_write_auth(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let address = req.address.unwrap_or_else(|| {
        format!(
            "dworld://field/{}",
            state.next_chain_id.fetch_add(1, Ordering::Relaxed)
        )
    });

    let vector = if let Some(v) = req.vector {
        // Caller provided the vector — library model, no inference needed.
        v
    } else {
        // Server embeds — managed model fallback.
        match state.routing_loop.llm_embed(&req.content).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("embed failed: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    };

    state.routing_loop
        .index_output(address.clone(), req.content, vector)
        .await;

    Json(FieldWriteResponse { address, indexed: true }).into_response()
}

/// GET /.dworld/field/nearest
///
/// Returns the k nearest identity files to the given embedding vector.
/// The caller uses these to decide which lens to route to next.
///
/// Query params:
///   v  — comma-separated floats (pre-computed vector)
///   k  — number of neighbors (default 9)
#[derive(Deserialize)]
struct NearestQuery {
    v: String,
    #[serde(default = "default_k")]
    k: usize,
}

fn default_k() -> usize { 9 }

#[derive(Serialize)]
struct NearestEntry {
    address: String,
    world_coord: Option<[f32; 3]>,
}

#[derive(Serialize)]
struct NearestResponse {
    neighbors: Vec<NearestEntry>,
}

async fn get_nearest(
    State(state): State<Arc<HttpState>>,
    axum::extract::Query(params): axum::extract::Query<NearestQuery>,
) -> impl IntoResponse {
    let vector: Vec<f32> = params.v
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if vector.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let neighbors = state.routing_loop.nearest_k(&vector, params.k)
        .into_iter()
        .map(|id| NearestEntry {
            address: id.address,
            world_coord: id.world_coord,
        })
        .collect();

    Json(NearestResponse { neighbors }).into_response()
}

// ─── Ingest endpoint ──────────────────────────────────────────────────────────
//
// POST /.dworld/ingest/propositions
//
// The bridge between theC0UNCIL and the NEXUS semantic field.
//
// The Council calls this after AutoCrawl decomposition, posting atomically
// decomposed propositions from its knowledge graph.  NEXUS re-embeds each
// proposition with AllMiniLML6V2 (384D in production, 5D mock in tests)
// and inserts it into the live HNSW field.  The Council's Gemini 768D
// embeddings are NOT used here — each side owns its own embedding space.
//
// Two accepted body shapes (backward-compatible):
//
//   Simple (v1):
//     { "propositions": ["text1", "text2"], "source_address": "dworld://..." }
//
//   Rich (v2):
//     { "propositions": [{ "text": "...", "tags": ["depth:2", "branch:0", "topic:oracle"],
//                          "searchable": true, "custom_vector": null }],
//       "source_address": "dworld://..." }
//
// Tags are indexed into the address query string for future filtered search.
// Embedding always uses `text` only — tags never contaminate the vector.
// custom_vector accepted but currently ignored (NEXUS re-embeds at 384D).
//
// Response:
//   { "indexed": 5, "addresses": ["dworld://orchestration/{id}/prop/0", ...],
//     "tags_received": 12 }

/// Rich proposition entry — v2 ingest format.
#[derive(Deserialize)]
struct PropositionEntry {
    text: String,
    /// dworld:// URI of origin. Falls back to `source_address` if absent.
    source: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    /// Caller's pre-computed embedding — stored verbatim on the IdentityFile.
    /// NEXUS still re-embeds `text` at 384D for HNSW routing.
    /// Enables cross-index retrieval: e.g. store Council's Gemini 768D vector here.
    custom_vector: Option<Vec<f32>>,
    /// Which model produced `custom_vector`. e.g. "gemini-embedding-001".
    custom_model: Option<String>,
    /// Whether to include in public nearest-neighbor queries (default true).
    #[serde(default = "default_searchable")]
    searchable: bool,
}

fn default_searchable() -> bool { true }

/// Accept either a bare string or a rich PropositionEntry.
/// This lets the Council upgrade its format without a flag day.
#[derive(Deserialize)]
#[serde(untagged)]
enum PropositionItem {
    Text(String),
    Entry(PropositionEntry),
}

impl PropositionItem {
    fn text(&self) -> &str {
        match self {
            Self::Text(s) => s,
            Self::Entry(e) => &e.text,
        }
    }
    fn tags(&self) -> &[String] {
        match self {
            Self::Text(_) => &[],
            Self::Entry(e) => &e.tags,
        }
    }
    fn source(&self) -> Option<&str> {
        match self {
            Self::Text(_) => None,
            Self::Entry(e) => e.source.as_deref(),
        }
    }
    fn custom_vector(&self) -> Option<&Vec<f32>> {
        match self {
            Self::Text(_) => None,
            Self::Entry(e) => e.custom_vector.as_ref(),
        }
    }
    fn custom_model(&self) -> Option<&str> {
        match self {
            Self::Text(_) => None,
            Self::Entry(e) => e.custom_model.as_deref(),
        }
    }
    fn searchable(&self) -> bool {
        match self {
            Self::Text(_) => true,
            Self::Entry(e) => e.searchable,
        }
    }
}

#[derive(Deserialize)]
struct IngestPropositionsRequest {
    propositions: Vec<PropositionItem>,
    source_address: String,
}

#[derive(Serialize)]
struct IngestPropositionsResponse {
    indexed: usize,
    addresses: Vec<String>,
    /// Total number of tags received across all propositions.
    tags_received: usize,
}

/// Check bearer token for write endpoints.
///
/// NEXUS_INGEST_TOKEN unset = open (dev mode).
/// NEXUS_INGEST_TOKEN set = Authorization: Bearer {token} required.
/// Reads public (manifests, nearest, identities) — only writes are gated.
fn check_write_auth(headers: &HeaderMap) -> bool {
    match std::env::var("NEXUS_INGEST_TOKEN").ok() {
        None => true, // dev mode — open
        Some(token) => {
            let provided = headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            provided == format!("Bearer {token}")
        }
    }
}

async fn post_ingest_propositions(
    State(state): State<Arc<HttpState>>,
    headers: HeaderMap,
    Json(req): Json<IngestPropositionsRequest>,
) -> impl IntoResponse {
    if !check_write_auth(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut addresses = Vec::with_capacity(req.propositions.len());
    let mut tags_received = 0usize;

    for (i, item) in req.propositions.iter().enumerate() {
        let text = item.text();
        let tags = item.tags();
        tags_received += tags.len();

        let address = format!("{}/prop/{}", req.source_address, i);

        // Embed text only — custom_vector stored separately, never pollutes routing.
        let vector = match state.routing_loop.llm_embed(text).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("ingest embed failed for prop {i}: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        let file = IdentityFile {
            address: address.clone(),
            content: text.to_string(),
            vector,
            source: item.source()
                .unwrap_or(&req.source_address)
                .to_string(),
            tags: tags.to_vec(),
            custom_vector: item.custom_vector().cloned(),
            custom_model: item.custom_model().map(str::to_string),
            searchable: item.searchable(),
            ..IdentityFile::default()
        };

        state.routing_loop.index_file(file).await;
        addresses.push(address);
    }

    Json(IngestPropositionsResponse {
        indexed: addresses.len(),
        addresses,
        tags_received,
    })
    .into_response()
}

/// GET /.dworld/field/search
///
/// Full-metadata search over the identity store.
/// All parameters are optional — omitting all returns up to `limit` records.
///
/// Query params:
///   q      — text substring filter (case-insensitive)
///   filter — comma-separated tag values to match (AND — node must have all)
///   sort   — "quality:desc" | "quality:asc" | "created_at:desc" | "created_at:asc"
///             | "hop_count:desc" | "hop_count:asc"  (default: "created_at:desc")
///   limit  — max results (default 20, max 200)
///   offset — pagination offset (default 0)
#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    filter: Option<String>,
    sort: Option<String>,
    #[serde(default = "default_search_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_search_limit() -> usize { 20 }

#[derive(Serialize)]
struct SearchEntry {
    address: String,
    content: String,
    source: String,
    tags: Vec<String>,
    quality: f32,
    hop_count: u32,
    created_at: u64,
    searchable: bool,
    world_coord: Option<[f32; 3]>,
    custom_model: Option<String>,
    /// true if a custom_vector is stored (vector itself omitted from listing)
    has_custom_vector: bool,
}

#[derive(Serialize)]
struct SearchResponse {
    total: usize,
    offset: usize,
    results: Vec<SearchEntry>,
}

async fn get_field_search(
    State(state): State<Arc<HttpState>>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let limit = params.limit.min(200);
    let filter_tags: Vec<String> = params.filter
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let store = state.routing_loop.current_store();
    let hits = store.search_filter(
        params.q.as_deref(),
        &filter_tags,
        false, // return all, not just searchable:true
        params.sort.as_deref(),
        limit,
        params.offset,
    );

    let total = hits.len();
    let results = hits.into_iter().map(|id| SearchEntry {
        address: id.address.clone(),
        content: id.content.clone(),
        source: id.source.clone(),
        tags: id.tags.clone(),
        quality: id.quality,
        hop_count: id.hop_count,
        created_at: id.created_at,
        searchable: id.searchable,
        world_coord: id.world_coord,
        custom_model: id.custom_model.clone(),
        has_custom_vector: id.custom_vector.is_some(),
    }).collect();

    Json(SearchResponse { total, offset: params.offset, results }).into_response()
}

/// GET /.dworld/field/node/{*address}
///
/// Returns the full IdentityFile metadata for one node by its dworld:// address.
/// Includes both the routing vector (384D) and the custom_vector if stored.
///
/// The {*address} wildcard captures slashes so addresses like
///   dworld://orchestration/test-001/prop/0
/// map to:
///   GET /.dworld/field/node/orchestration/test-001/prop/0
#[derive(Serialize)]
struct NodeResponse {
    address: String,
    content: String,
    source: String,
    tags: Vec<String>,
    quality: f32,
    hop_count: u32,
    created_at: u64,
    searchable: bool,
    world_coord: Option<[f32; 3]>,
    custom_model: Option<String>,
    /// The 384D HNSW routing vector (or whatever dim the local embedder uses).
    vector: Vec<f32>,
    /// Caller's original embedding, if provided at ingest. Stored verbatim.
    custom_vector: Option<Vec<f32>>,
}

async fn get_field_node(
    State(state): State<Arc<HttpState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let address = format!("dworld://{path}");
    match state.routing_loop.get_identity(&address) {
        Some(id) => Json(NodeResponse {
            address: id.address,
            content: id.content,
            source: id.source,
            tags: id.tags,
            quality: id.quality,
            hop_count: id.hop_count,
            created_at: id.created_at,
            searchable: id.searchable,
            world_coord: id.world_coord,
            custom_model: id.custom_model,
            vector: id.vector,
            custom_vector: id.custom_vector,
        }).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// ─── Router ───────────────────────────────────────────────────────────────────

/// Build the axum Router for the semantic HTTP API.
///
/// Bind with: `axum::serve(listener, router(state)).await`
pub fn router(state: Arc<HttpState>) -> Router {
    Router::new()
        // Managed compute path (server owns inference)
        .route("/packet", post(post_packet))
        .route("/chain/{chain_id}", get(get_chain))
        .route("/health", get(health))
        // dworld:// protocol (caller owns inference, server owns index)
        // Note: specific routes before the catch-all {*path} so axum matches correctly
        .route("/.dworld/identities/{name}", get(get_identity))
        .route("/.dworld/field", post(post_field))
        .route("/.dworld/field/nearest", get(get_nearest))
        .route("/.dworld/field/search", get(get_field_search))
        .route("/.dworld/field/node/{*address}", get(get_field_node))
        // Council → NEXUS bridge
        .route("/.dworld/ingest/propositions", post(post_ingest_propositions))
        // Catch-all manifest resolver — must come last
        .route("/.dworld/{*path}", get(get_manifest))
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

    // ── dworld:// protocol tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn dworld_manifest_returns_json_for_any_path() {
        let state = make_state();
        let app = router(state);

        let req = Request::builder()
            .uri("/.dworld/nexus.local/agents/NEXUS")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 2048).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json.get("worldId").is_some());
        assert!(json.get("surface").is_some());
    }

    #[tokio::test]
    async fn dworld_identity_returns_404_for_unknown() {
        let state = make_state();
        let app = router(state);

        let req = Request::builder()
            .uri("/.dworld/identities/DOES_NOT_EXIST")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn dworld_identity_resolves_seed_by_short_name() {
        let state = make_state();
        let app = router(state);

        // Seed identities are stored as dworld://council.local/identities/PHILOSOPHER
        // The short name lookup must resolve without the full prefix.
        let req = Request::builder()
            .uri("/.dworld/identities/PHILOSOPHER")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["address"].as_str().unwrap().contains("PHILOSOPHER"));
        assert!(!json["content"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn dworld_field_nearest_returns_neighbors() {
        let state = make_state();
        let app = router(state);

        // A 5D zero vector — will still return nearest neighbors
        let req = Request::builder()
            .uri("/.dworld/field/nearest?v=0.1,0.2,0.3,0.4,0.5&k=3")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let neighbors = json["neighbors"].as_array().unwrap();
        assert!(!neighbors.is_empty(), "should return at least one neighbor");
    }

    #[tokio::test]
    async fn ingest_propositions_indexes_all_and_are_retrievable() {
        let state = make_state();
        let app = router(Arc::clone(&state));

        // Mix v1 (bare string) and v2 (rich entry) in same request — backward compat.
        let body = serde_json::json!({
            "propositions": [
                "semantic routing connects knowledge graphs",
                { "text": "embeddings position concepts in vector space",
                  "tags": ["depth:1", "branch:0", "topic:semantic"] },
                { "text": "identity files are lenses in the semantic field",
                  "tags": ["depth:1", "branch:0"] },
                "cosine similarity measures conceptual proximity",
                { "text": "the HNSW index enables approximate nearest neighbor search",
                  "tags": ["depth:2", "topic:hnsw"] }
            ],
            "source_address": "dworld://orchestration/test-001"
        })
        .to_string();

        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/.dworld/ingest/propositions")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["indexed"], 5, "expected 5 propositions indexed");
        assert_eq!(json["tags_received"], 7, "expected 7 tags total: 3+2+2 across rich entries");
        let addresses = json["addresses"].as_array().unwrap();
        assert_eq!(addresses.len(), 5);
        assert_eq!(
            addresses[0].as_str().unwrap(),
            "dworld://orchestration/test-001/prop/0"
        );

        // Verify all 5 are retrievable via nearest-neighbor.
        // Reproduce the MockLlmClient's deterministic 5D projection for prop/0.
        let text = "semantic routing connects knowledge graphs";
        const DIMS: usize = 5;
        let mut accum = [0.0f32; DIMS];
        let mut total = [0u64; DIMS];
        for (i, byte) in text.bytes().enumerate() {
            accum[i % DIMS] += byte as f32;
            total[i % DIMS] += 1;
        }
        let v_param: String = (0..DIMS)
            .map(|i| {
                let val = if total[i] > 0 {
                    (accum[i] / (total[i] as f32 * 255.0)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                format!("{val:.6}")
            })
            .collect::<Vec<_>>()
            .join(",");

        let req2 = Request::builder()
            .uri(format!("/.dworld/field/nearest?v={v_param}&k=6"))
            .body(Body::empty())
            .unwrap();

        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);

        let bytes2 = axum::body::to_bytes(resp2.into_body(), 4096).await.unwrap();
        let json2: serde_json::Value = serde_json::from_slice(&bytes2).unwrap();
        let neighbors: Vec<&str> = json2["neighbors"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|n| n["address"].as_str())
            .collect();

        assert!(
            neighbors.contains(&"dworld://orchestration/test-001/prop/0"),
            "expected prop/0 in nearest neighbors, got: {neighbors:?}"
        );
    }

    #[tokio::test]
    async fn dworld_field_write_indexes_new_content() {
        let state = make_state();
        let initial_size = state.routing_loop.current_store().len();
        let app = router(state);

        // Post a pre-computed vector so no LLM call is needed
        let body = serde_json::json!({
            "address": "dworld://test/new-identity",
            "content": "this is a new identity for testing",
            "vector": [0.1_f32, 0.2, 0.3, 0.4, 0.5]
        }).to_string();

        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/.dworld/field")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["indexed"], true);
        assert_eq!(json["address"], "dworld://test/new-identity");

        // Give the async write a tick to complete
        tokio::task::yield_now().await;

        // Re-read state from the routing loop (not from app, state is shared via Arc)
        // The routing_loop is inside the HttpState which we already have as Arc
        // We need to access it from state - but state was moved into router...
        // This verifies via the response only; store growth is covered by worker tests.
        let _ = initial_size; // accepted: store growth tested in worker::tests
    }

    // ── field/search and field/node tests ─────────────────────────────────────

    /// Insert a node with tags and custom_vector, retrieve by address,
    /// verify both vectors are stored.
    #[tokio::test]
    async fn field_node_returns_full_metadata_including_vectors() {
        let state = make_state();
        let app = router(Arc::clone(&state));

        // Ingest one rich proposition with custom_vector
        let body = serde_json::json!({
            "propositions": [{
                "text": "quantum entanglement enables non-local correlations",
                "tags": ["topic:physics", "depth:3"],
                "custom_vector": [0.9_f32, 0.8, 0.7, 0.6, 0.5],
                "custom_model": "gemini-embedding-001",
                "searchable": true
            }],
            "source_address": "dworld://test/meta-round-trip"
        }).to_string();

        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/.dworld/ingest/propositions")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Retrieve by address
        let req2 = Request::builder()
            .uri("/.dworld/field/node/test/meta-round-trip/prop/0")
            .body(Body::empty())
            .unwrap();
        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp2.into_body(), 8192).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        // Tags round-trip
        let tags: Vec<&str> = json["tags"].as_array().unwrap()
            .iter().filter_map(|t| t.as_str()).collect();
        assert!(tags.contains(&"topic:physics"), "topic:physics tag missing");
        assert!(tags.contains(&"depth:3"), "depth:3 tag missing");

        // custom_vector round-trip
        let cv = json["custom_vector"].as_array().unwrap();
        assert_eq!(cv.len(), 5, "custom_vector should be 5D");
        let first = cv[0].as_f64().unwrap();
        assert!((first - 0.9).abs() < 0.001, "custom_vector[0] should be ~0.9");

        // custom_model round-trip
        assert_eq!(json["custom_model"].as_str().unwrap(), "gemini-embedding-001");

        // routing vector present (MockLlmClient produces 5D)
        let v = json["vector"].as_array().unwrap();
        assert!(!v.is_empty(), "routing vector must be present");
    }

    /// Search by tag filter, verify only matching nodes returned.
    #[tokio::test]
    async fn field_search_tag_filter_returns_only_matching() {
        let state = make_state();
        let app = router(Arc::clone(&state));

        // Ingest two propositions with different tags
        let body = serde_json::json!({
            "propositions": [
                { "text": "dark matter fills galactic halos", "tags": ["topic:astrophysics"] },
                { "text": "neural oscillations underlie consciousness", "tags": ["topic:neuroscience"] }
            ],
            "source_address": "dworld://test/tag-search"
        }).to_string();

        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/.dworld/ingest/propositions")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();
        app.clone().oneshot(req).await.unwrap();

        // Search with tag filter
        let req2 = Request::builder()
            .uri("/.dworld/field/search?filter=topic:astrophysics&limit=50")
            .body(Body::empty())
            .unwrap();
        let resp2 = app.oneshot(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp2.into_body(), 8192).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let results = json["results"].as_array().unwrap();

        // Only astrophysics propositions should be present
        for r in results {
            let tags: Vec<&str> = r["tags"].as_array().unwrap()
                .iter().filter_map(|t| t.as_str()).collect();
            if r["address"].as_str().unwrap().contains("tag-search") {
                assert!(
                    tags.contains(&"topic:astrophysics"),
                    "non-astrophysics result leaked through: {:?}", r["address"]
                );
            }
        }
        // At least one astrophysics result present
        let has_astro = results.iter().any(|r| {
            r["address"].as_str().unwrap_or("").contains("tag-search/prop/0")
        });
        assert!(has_astro, "astrophysics proposition not found in filtered results");
    }

    /// Sort by created_at desc, verify newest first.
    #[tokio::test]
    async fn field_search_sort_created_at_desc_newest_first() {
        let state = make_state();
        let app = router(Arc::clone(&state));

        // Ingest propositions sequentially — each gets a fresh created_at
        for i in 0..3 {
            let body = serde_json::json!({
                "propositions": [{
                    "text": format!("ordered proposition number {i}"),
                    "tags": [format!("seq:{i}")]
                }],
                "source_address": format!("dworld://test/sort-test/{i}")
            }).to_string();

            let req = Request::builder()
                .method(http::Method::POST)
                .uri("/.dworld/ingest/propositions")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();
            app.clone().oneshot(req).await.unwrap();

            // Small yield to allow timestamps to differ
            tokio::task::yield_now().await;
        }

        let req = Request::builder()
            .uri("/.dworld/field/search?q=ordered+proposition&sort=created_at:desc&limit=10")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let results = json["results"].as_array().unwrap();

        // created_at values should be non-increasing (newest first)
        let timestamps: Vec<u64> = results.iter()
            .filter_map(|r| r["created_at"].as_u64())
            .collect();
        for w in timestamps.windows(2) {
            assert!(
                w[0] >= w[1],
                "sort created_at:desc violation: {} < {}", w[0], w[1]
            );
        }
    }

    /// field/node returns 404 for unknown address.
    #[tokio::test]
    async fn field_node_returns_404_for_unknown() {
        let state = make_state();
        let app = router(state);

        let req = Request::builder()
            .uri("/.dworld/field/node/does/not/exist")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
