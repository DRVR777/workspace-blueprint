//! The routing loop — bone 1b.
//!
//! Router::route is the five-step atom of the semantic layer:
//!
//!   1. Embed current packet data  → query vector
//!   2. Nearest identity in store  → the lens
//!   3. One LLM call               → output string
//!   4. push_hop on the packet     → chain grows
//!   5. Embed output               → new packet data position
//!      → RouteResult::Continue or ::Terminal
//!
//! The router is stateless between calls. No session. No memory.
//! Everything the loop needs is in the packet. Everything the packet
//! accumulates is in meta.

use std::sync::Arc;
use std::time::SystemTime;
use nexus_core::types::{SemanticPacket, PacketData, TimestampMs};
use crate::identity::IdentityStore;
use crate::llm::LlmClient;

/// Output of one routing step.
#[derive(Debug)]
pub enum RouteResult {
    /// The packet should continue. The inner packet is the updated version
    /// with the new hop recorded and `data` replaced by the output text.
    Continue(SemanticPacket),
    /// The packet has reached a terminal condition. Inner string is the
    /// final output. The packet with full hop chain is also returned for
    /// event log recording.
    Terminal { output: String, packet: SemanticPacket },
}

/// The semantic router.
///
/// Holds a reference to the identity store and the LLM client.
/// Immutable after construction — safe to share across worker threads via Arc.
pub struct Router {
    pub store: Arc<IdentityStore>,
    pub llm:   Arc<dyn LlmClient>,
}

impl Router {
    pub fn new(store: Arc<IdentityStore>, llm: Arc<dyn LlmClient>) -> Self {
        Self { store, llm }
    }

    /// Execute one routing step on `packet`.
    ///
    /// Mutates the packet (adds a hop record), then returns RouteResult.
    /// The caller is responsible for requeueing on Continue or recording on Terminal.
    pub async fn route(&self, mut packet: SemanticPacket) -> Result<RouteResult, RouterError> {
        // ── Step 1: embed the packet's current data ───────────────────────
        let query_text = data_to_query_text(&packet.data);
        let query_vector = self.llm.embed(&query_text).await
            .map_err(|e| RouterError::Embed(e.to_string()))?;

        // ── Step 2: nearest identity file ────────────────────────────────
        let identity = self.store.nearest(&query_vector)
            .ok_or(RouterError::EmptyStore)?;

        // ── Step 3: one LLM call ──────────────────────────────────────────
        let output = self.llm.complete(&identity.content, &packet).await
            .map_err(|e| RouterError::Complete(e.to_string()))?;

        // ── Step 4: push_hop ─────────────────────────────────────────────
        let now_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as TimestampMs;

        packet.push_hop(now_ms, identity.address.clone(), Some(identity.vector));

        // ── Step 5: terminal check, then embed output ────────────────────
        if packet.terminal || packet.depth_exceeded() {
            return Ok(RouteResult::Terminal { output, packet });
        }

        // Embed the output to form the next packet's data position.
        // The output text becomes the new packet data. The hop chain carries
        // the history. The next routing step will embed this output and find
        // the nearest identity to *it* — the packet navigates by what it has become.
        let new_data = PacketData::Text(output.clone());

        // Check if the output signals termination (simple heuristic: very short
        // outputs from the SYNTHESIZER or OBSERVER often indicate convergence).
        // Phase 1+: replace with a quality-score threshold check.
        let is_terminal = packet.depth_exceeded()
            || output.len() < 20  // very short = likely converged
            || output.to_lowercase().contains("[terminal]");

        if is_terminal {
            packet.terminal = true;
            return Ok(RouteResult::Terminal { output, packet });
        }

        packet.data = new_data;
        Ok(RouteResult::Continue(packet))
    }
}

/// Extract a query string from the packet's current data.
/// Used to compute the embedding that selects the next identity.
fn data_to_query_text(data: &PacketData) -> String {
    match data {
        PacketData::Text(s)                          => s.clone(),
        PacketData::Program { lang, source, .. }     => format!("{lang}: {source}"),
        PacketData::Spatial { address, .. }          => format!("spatial {address}"),
        PacketData::Signal { kind, value, .. }       => format!("signal {kind} {value:?}"),
        PacketData::Identity { address, content, .. }=> format!("{address}: {content}"),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("embed failed: {0}")]
    Embed(String),
    #[error("complete failed: {0}")]
    Complete(String),
    #[error("identity store is empty")]
    EmptyStore,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::seed_identities;
    use crate::llm::MockLlmClient;
    use nexus_core::types::SemanticPacket;

    fn make_router() -> Router {
        let store = Arc::new(IdentityStore::build(seed_identities()));
        let llm   = Arc::new(MockLlmClient::new());
        Router::new(store, llm)
    }

    #[tokio::test]
    async fn route_produces_a_hop() {
        let router = make_router();
        let packet = SemanticPacket::new(
            1, 1,
            PacketData::Text("What is the deepest structure of computation?".into()),
            "dworld://test/".into(),
        );
        assert!(packet.meta.is_empty());

        let result = router.route(packet).await.unwrap();
        match result {
            RouteResult::Continue(p) | RouteResult::Terminal { packet: p, .. } => {
                assert_eq!(p.meta.len(), 1);
                assert!(p.meta[0].identity.starts_with("dworld://council.local/identities/"));
                assert!(p.meta[0].vector.is_some());
                assert!(p.meta[0].timestamp_ms > 0);
            }
        }
    }

    #[tokio::test]
    async fn route_selects_some_identity_for_code_packet() {
        // The mock LLM embeds by byte frequency (not semantics), so we don't
        // assert which specific identity wins — only that one was selected and
        // the hop was recorded. Semantic identity selection is proven in
        // identity::tests::nearest_returns_closest_identity which uses
        // hand-crafted vectors against the real cosine distance function.
        let router = make_router();
        let packet = SemanticPacket::new(
            2, 2,
            PacketData::Program {
                lang: "rust".into(),
                source: "fn sort(v: &mut Vec<i32>) { v.sort(); }".into(),
                entrypoint: None,
            },
            "dworld://test/".into(),
        );

        let result = router.route(packet).await.unwrap();
        let hop = match &result {
            RouteResult::Continue(p) => &p.meta[0],
            RouteResult::Terminal { packet: p, .. } => &p.meta[0],
        };
        assert!(hop.identity.starts_with("dworld://council.local/identities/"));
        assert!(hop.vector.is_some());
    }

    #[tokio::test]
    async fn chain_terminates_at_depth_limit() {
        let router = make_router();
        let mut packet = SemanticPacket::new(
            3, 3,
            PacketData::Text("keep going".into()),
            "dworld://test/".into(),
        );
        packet.depth_limit = 3;

        // Run until terminal
        let mut steps = 0usize;
        loop {
            match router.route(packet).await.unwrap() {
                RouteResult::Terminal { .. } => break,
                RouteResult::Continue(p) => {
                    packet = p;
                    steps += 1;
                    assert!(steps <= 10, "loop did not terminate");
                }
            }
        }
        // Should have terminated at or before depth_limit
        assert!(steps <= 3);
    }

    #[tokio::test]
    async fn each_route_call_increments_llm_call_count() {
        let store = Arc::new(IdentityStore::build(seed_identities()));
        let llm   = Arc::new(MockLlmClient::new());
        let router = Router::new(store, llm.clone());

        let packet = SemanticPacket::new(
            4, 4,
            PacketData::Text("test".into()),
            "dworld://test/".into(),
        );
        router.route(packet).await.unwrap();
        assert_eq!(llm.call_count(), 1);
    }
}
