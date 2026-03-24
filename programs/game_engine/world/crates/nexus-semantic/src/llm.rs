//! LLM client trait and implementations.
//!
//! The routing loop calls exactly two LLM operations:
//!   embed(text) → [f32; DIMS]   — positions the packet in the field
//!   complete(identity, packet) → String — the one call per hop
//!
//! MockLlmClient: deterministic, no network, used in all tests.
//! AnthropicClient: real HTTP to claude-sonnet-4-6 via the Messages API.
//!
//! When the real embedding model goes in, `embed` returns [f32; 768].
//! The interface is unchanged — only DIMS changes.

use std::sync::atomic::{AtomicU64, Ordering};
use nexus_core::types::SemanticPacket;
use crate::identity::DIMS;

/// Contract between the routing loop and any LLM backend.
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// Embed `text` into a [f32; DIMS] vector representing its position
    /// in the semantic field.
    ///
    /// Phase 0: mock returns a deterministic projection.
    /// Phase 1+: calls the embedding endpoint of the configured model.
    async fn embed(&self, text: &str) -> Result<[f32; DIMS], LlmError>;

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

/// Deterministic LLM client for tests and bone 1c proof.
///
/// embed: projects the text's byte sum into [0, 1]^DIMS using a simple hash.
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
    async fn embed(&self, text: &str) -> Result<[f32; DIMS], LlmError> {
        // Deterministic projection: hash each byte into one of the DIMS axes.
        // Two identical texts produce identical vectors. Similar texts produce
        // similar vectors by construction (shared byte distribution).
        let mut accum = [0.0f32; DIMS];
        let mut total = [0u64; DIMS];
        for (i, byte) in text.bytes().enumerate() {
            let axis = i % DIMS;
            accum[axis] += byte as f32;
            total[axis] += 1;
        }
        let mut result = [0.0f32; DIMS];
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

// ─── Anthropic client ────────────────────────────────────────────────────────

/// Real LLM client backed by the Anthropic Messages API.
///
/// Requires ANTHROPIC_API_KEY environment variable.
/// Model: claude-sonnet-4-6.
///
/// embed: Phase 0 uses the same hash projection as MockLlmClient.
/// Phase 1+: replace with a call to the embeddings endpoint.
pub struct AnthropicClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?;
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
            model: "claude-sonnet-4-6".to_string(),
        })
    }
}

#[async_trait::async_trait]
impl LlmClient for AnthropicClient {
    async fn embed(&self, text: &str) -> Result<[f32; DIMS], LlmError> {
        // Phase 0: same hash projection as mock.
        // Phase 1+: replace with embeddings API call.
        let mock = MockLlmClient::new();
        mock.embed(text).await
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
