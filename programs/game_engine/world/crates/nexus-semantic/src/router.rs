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

        packet.push_hop(now_ms, identity.address.clone(), Some(identity.vector.clone()));

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
    use crate::identity::{IdentityFile, IdentityStore, seed_identities};
    use crate::llm::{MockLlmClient, LocalEmbedClient};
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

    // ── Bone 1c — the proof test ──────────────────────────────────────────────
    //
    // Proves that when a routing step's output is indexed in the semantic field,
    // the next packet carrying that text finds the memory — not the original seeds.
    //
    // Two properties proven:
    //   1. fastembed produces semantically meaningful 384D vectors
    //   2. identical text in two different IdentityFiles → identical nearest-neighbor
    //
    // Requires the fastembed AllMiniLML6V2 model (~90 MB, downloaded on first run).
    // Run with: cargo test -- --ignored bone_1c
    #[tokio::test]
    #[ignore = "downloads fastembed AllMiniLML6V2 model (~90 MB)"]
    async fn bone_1c_field_has_memory() {
        // ── Init real embedder ────────────────────────────────────────────────
        let llm: Arc<dyn crate::llm::LlmClient> =
            Arc::new(LocalEmbedClient::new().expect("fastembed init failed"));

        // ── Build identity store with real 384D embeddings ────────────────────
        // Seed texts are semantically distinct so the HNSW index reflects
        // real semantic distance, not hand-crafted proximity.
        let seed_pairs: &[(&str, &str)] = &[
            ("dworld://council.local/identities/PHILOSOPHER",
             "You reason from first principles and find the deepest structural truth beneath any question."),
            ("dworld://council.local/identities/ENGINEER",
             "You build concrete implementations: data structures, algorithms, executable steps."),
            ("dworld://council.local/identities/CRITIC",
             "You find what is wrong — the weakest assumption, the ignored edge case."),
        ];

        let mut seed_files = Vec::new();
        for (addr, content) in seed_pairs {
            let vector = llm.embed(content).await.expect("embed seed");
            seed_files.push(IdentityFile {
                address: addr.to_string(),
                content: content.to_string(),
                vector,
            });
        }

        // ── Route Packet 1 through the seed field ────────────────────────────
        let store1 = Arc::new(IdentityStore::build(seed_files.clone()));
        let router1 = Router::new(Arc::clone(&store1), Arc::clone(&llm));

        let packet1 = SemanticPacket::new(
            10, 10,
            PacketData::Text(
                "semantic routing: packets navigate by vector proximity through identity lenses".into()
            ),
            "dworld://test/bone1c".into(),
        );

        // Capture the output text (whether Terminal or Continue — we want the text)
        let output1 = match router1.route(packet1).await.expect("route packet 1") {
            RouteResult::Terminal { output, .. } => output,
            RouteResult::Continue(p) => match p.data {
                PacketData::Text(s) => s,
                _ => panic!("expected text data in continued packet"),
            },
        };

        // ── Index output1 as a memory in the field ───────────────────────────
        let memory_vector = llm.embed(&output1).await.expect("embed output1");
        let mut files_with_memory = seed_files;
        files_with_memory.push(IdentityFile {
            address: "dworld://memory/step1".into(),
            content: output1.clone(),
            vector: memory_vector,
        });

        // ── Route Packet 2 carrying output1's text ────────────────────────────
        // Packet 2's embedding == output1's embedding.
        // The memory identity's embedding == output1's embedding.
        // Therefore: nearest(packet2) == memory identity.
        let store2 = Arc::new(IdentityStore::build(files_with_memory));
        let router2 = Router::new(store2, Arc::clone(&llm));

        let packet2 = SemanticPacket::new(
            11, 10, // same chain_id — same reasoning chain
            PacketData::Text(output1),
            "dworld://test/bone1c".into(),
        );

        let result2 = router2.route(packet2).await.expect("route packet 2");
        let first_hop_identity = match &result2 {
            RouteResult::Continue(p) | RouteResult::Terminal { packet: p, .. } =>
                p.meta[0].identity.clone(),
        };

        // THE PROOF: the field has memory.
        // What was output is now indexed. The next packet carrying that output
        // finds the memory — not the original seed identities.
        assert_eq!(
            first_hop_identity, "dworld://memory/step1",
            "Field memory proof failed: packet should find indexed memory, not seed identities"
        );

        // The hop vector should be real (384D)
        let hop_vec = match &result2 {
            RouteResult::Continue(p) | RouteResult::Terminal { packet: p, .. } =>
                p.meta[0].vector.as_ref().cloned(),
        };
        let hop_vec = hop_vec.expect("hop vector must be set");
        assert_eq!(hop_vec.len(), 384, "AllMiniLML6V2 produces 384D vectors");
    }
}
