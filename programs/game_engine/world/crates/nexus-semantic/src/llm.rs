//! LLM client trait and implementations.
//!
//! The routing loop calls exactly two LLM operations:
//!   embed(text) → Vec<f32>  — positions the packet in the field
//!   complete(identity, packet) → String — the one call per hop
//!
//! MockLlmClient: deterministic, no network, used in unit tests.
//! LocalEmbedClient: fastembed (AllMiniLML6V2, 384D) + mock complete — used in bone 1c proof.
//! AnthropicClient: fastembed embed + real HTTP to claude-sonnet-4-6 — used in production.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use nexus_core::types::SemanticPacket;

/// Contract between the routing loop and any LLM backend.
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// Embed `text` into a `Vec<f32>` representing its position in the semantic field.
    ///
    /// Phase 0 (mock): deterministic byte-frequency projection.
    /// Phase 1+ (fastembed): AllMiniLML6V2 → 384D dense vector.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, LlmError>;

    /// Embed multiple texts in one call. Default: sequential (correct for all backends).
    /// Override in fastembed backends for true batching — fastembed processes Vec<S>
    /// in parallel internally, ~3-5× faster than sequential for n > 10.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, LlmError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Make one completion call: inject `identity_content` as the system
    /// identity and the packet's current data + hop chain as the user message.
    ///
    /// Returns the raw output string. The router embeds it separately.
    async fn complete(
        &self,
        identity_content: &str,
        packet: &SemanticPacket,
    ) -> Result<String, LlmError>;
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },
    #[error("Serialization error: {0}")]
    Serde(String),
}

// ─── Mock client ─────────────────────────────────────────────────────────────

/// Deterministic LLM client for unit tests.
///
/// embed: projects the text's byte frequency into [0, 1]^5 using a simple hash.
///        Output is always 5-dimensional regardless of input.
/// complete: returns "[IDENTITY_KEYWORD]: [packet_data_preview]".
///
/// The mock's output is semantically meaningless but structurally correct —
/// it produces a string that the router can embed and push_hop on.
pub struct MockLlmClient {
    call_count: AtomicU64,
}

impl MockLlmClient {
    pub fn new() -> Self {
        Self { call_count: AtomicU64::new(0) }
    }

    /// How many complete() calls have been made.
    pub fn call_count(&self) -> u64 {
        self.call_count.load(Ordering::Relaxed)
    }
}

impl Default for MockLlmClient {
    fn default() -> Self { Self::new() }
}

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        // Deterministic 5D projection: hash each byte into one of 5 axes.
        // Two identical texts produce identical vectors.
        const DIMS: usize = 5;
        let mut accum = [0.0f32; DIMS];
        let mut total = [0u64; DIMS];
        for (i, byte) in text.bytes().enumerate() {
            let axis = i % DIMS;
            accum[axis] += byte as f32;
            total[axis] += 1;
        }
        let mut result = vec![0.0f32; DIMS];
        for i in 0..DIMS {
            result[i] = if total[i] > 0 {
                (accum[i] / (total[i] as f32 * 255.0)).clamp(0.0, 1.0)
            } else {
                0.5
            };
        }
        Ok(result)
    }

    async fn complete(
        &self,
        identity_content: &str,
        packet: &SemanticPacket,
    ) -> Result<String, LlmError> {
        self.call_count.fetch_add(1, Ordering::Relaxed);

        // Extract the first line of the identity content as its keyword
        let identity_keyword = identity_content
            .lines()
            .find(|l| l.starts_with("You are "))
            .and_then(|l| l.split_whitespace().nth(2))
            .map(|s| s.trim_end_matches('.'))
            .unwrap_or("IDENTITY");

        let data_preview = match &packet.data {
            nexus_core::types::PacketData::Text(s) => s.chars().take(60).collect::<String>(),
            nexus_core::types::PacketData::Program { lang, .. } =>
                format!("[{lang} program]"),
            nexus_core::types::PacketData::Spatial { address, .. } =>
                format!("[spatial: {address}]"),
            nexus_core::types::PacketData::Signal { kind, value, .. } =>
                format!("[signal: {kind} = {value:?}]"),
            nexus_core::types::PacketData::Identity { address, .. } =>
                format!("[identity: {address}]"),
        };

        Ok(format!(
            "{identity_keyword} (hop {}): {data_preview}",
            packet.meta.len()
        ))
    }
}

// ─── Local embed client (fastembed) ──────────────────────────────────────────

/// Real embedding via fastembed (AllMiniLML6V2, 384D) + mock LLM completion.
///
/// Used for bone 1c proof test and any scenario that needs semantically
/// meaningful embeddings without an Anthropic API key.
///
/// On first use, downloads the AllMiniLML6V2 model (~90 MB) to the fastembed
/// cache directory (~/.cache/huggingface/hub).
pub struct LocalEmbedClient {
    embedder: Arc<Mutex<fastembed::TextEmbedding>>,
    mock: MockLlmClient,
}

impl LocalEmbedClient {
    /// Initialise the fastembed model. Downloads on first call.
    pub fn new() -> Result<Self, String> {
        let model = fastembed::TextEmbedding::try_new(
            fastembed::TextInitOptions::new(fastembed::EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(false),
        )
        .map_err(|e| format!("fastembed init: {e}"))?;
        Ok(Self {
            embedder: Arc::new(Mutex::new(model)),
            mock: MockLlmClient::new(),
        })
    }
}

#[async_trait::async_trait]
impl LlmClient for LocalEmbedClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        let embedder = self.embedder.clone();
        let text = text.to_string();
        tokio::task::spawn_blocking(move || {
            let mut guard = embedder.lock().unwrap();
            let embeddings = guard
                .embed(vec![text.as_str()], None)
                .map_err(|e| LlmError::Http(e.to_string()))?;
            Ok::<Vec<f32>, LlmError>(embeddings.into_iter().next().unwrap_or_default())
        })
        .await
        .map_err(|e| LlmError::Http(format!("spawn_blocking panicked: {e}")))?
    }

    /// Batch override — passes all texts to fastembed in one call.
    /// fastembed processes Vec<S> internally with parallelism, ~3-5× faster than sequential.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, LlmError> {
        let embedder = self.embedder.clone();
        let owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        tokio::task::spawn_blocking(move || {
            let mut guard = embedder.lock().unwrap();
            let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
            guard.embed(refs, None)
                .map_err(|e| LlmError::Http(e.to_string()))
        })
        .await
        .map_err(|e| LlmError::Http(format!("spawn_blocking panicked: {e}")))?
    }

    async fn complete(
        &self,
        identity_content: &str,
        packet: &SemanticPacket,
    ) -> Result<String, LlmError> {
        self.mock.complete(identity_content, packet).await
    }
}

// ─── Anthropic client ────────────────────────────────────────────────────────

/// Real LLM client backed by fastembed (embed) + Anthropic Messages API (complete).
///
/// Requires ANTHROPIC_API_KEY environment variable.
/// Model: claude-sonnet-4-6.
pub struct AnthropicClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
    embedder: Arc<Mutex<fastembed::TextEmbedding>>,
}

impl AnthropicClient {
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?;
        let model = fastembed::TextEmbedding::try_new(
            fastembed::TextInitOptions::new(fastembed::EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(false),
        )
        .map_err(|e| format!("fastembed init: {e}"))?;
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
            model: "claude-sonnet-4-6".to_string(),
            embedder: Arc::new(Mutex::new(model)),
        })
    }
}

#[async_trait::async_trait]
impl LlmClient for AnthropicClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        let embedder = self.embedder.clone();
        let text = text.to_string();
        tokio::task::spawn_blocking(move || {
            let mut guard = embedder.lock().unwrap();
            let embeddings = guard
                .embed(vec![text.as_str()], None)
                .map_err(|e| LlmError::Http(e.to_string()))?;
            Ok::<Vec<f32>, LlmError>(embeddings.into_iter().next().unwrap_or_default())
        })
        .await
        .map_err(|e| LlmError::Http(format!("spawn_blocking panicked: {e}")))?
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, LlmError> {
        let embedder = self.embedder.clone();
        let owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        tokio::task::spawn_blocking(move || {
            let mut guard = embedder.lock().unwrap();
            let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
            guard.embed(refs, None)
                .map_err(|e| LlmError::Http(e.to_string()))
        })
        .await
        .map_err(|e| LlmError::Http(format!("spawn_blocking panicked: {e}")))?
    }

    async fn complete(
        &self,
        identity_content: &str,
        packet: &SemanticPacket,
    ) -> Result<String, LlmError> {
        let user_message = build_user_message(packet);

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 1024,
            "system": identity_content,
            "messages": [
                { "role": "user", "content": user_message }
            ]
        });

        let response = self.http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        let status = response.status().as_u16();
        let body_text = response.text().await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        if status != 200 {
            return Err(LlmError::Api { status, body: body_text });
        }

        let parsed: serde_json::Value = serde_json::from_str(&body_text)
            .map_err(|e| LlmError::Serde(e.to_string()))?;

        parsed["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| LlmError::Serde("no text in response".into()))
    }
}

/// Build the user message from the packet's data + hop chain context.
/// The chain summary gives the LLM the full history of every prior hop.
fn build_user_message(packet: &SemanticPacket) -> String {
    let mut msg = String::new();

    // Chain context — what has already been produced
    if !packet.meta.is_empty() {
        msg.push_str("## Prior hops in this chain\n");
        for hop in &packet.meta {
            msg.push_str(&format!(
                "- Hop {}: {} (quality: {})\n",
                hop.hop,
                hop.identity,
                hop.quality.map(|q| format!("{q:.2}")).unwrap_or_else(|| "unscored".into()),
            ));
        }
        msg.push('\n');
    }

    // Current packet data
    msg.push_str("## Current packet\n");
    match &packet.data {
        nexus_core::types::PacketData::Text(s) => {
            msg.push_str(s);
        }
        nexus_core::types::PacketData::Program { lang, source, entrypoint } => {
            msg.push_str(&format!("Language: {lang}\n"));
            if let Some(ep) = entrypoint {
                msg.push_str(&format!("Entry: {ep}\n"));
            }
            msg.push_str(&format!("```{lang}\n{source}\n```"));
        }
        nexus_core::types::PacketData::Spatial { address, .. } => {
            msg.push_str(&format!("Spatial packet at: {address}"));
        }
        nexus_core::types::PacketData::Signal { kind, value, .. } => {
            msg.push_str(&format!("Signal: {kind} = {value:?}"));
        }
        nexus_core::types::PacketData::Identity { address, content, .. } => {
            msg.push_str(&format!("Identity file at {address}:\n{content}"));
        }
    }

    msg
}
