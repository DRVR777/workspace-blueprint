//! Identity file store with approximate nearest-neighbour retrieval.
//!
//! An identity file is the lens. The store is the ether — the collection
//! of all lenses suspended in the semantic field.
//!
//! Phase 0: vectors are [f32; 5] matching the knowledge-graph's 5D spec
//! (specificity, technicality, temporality, centrality, confidence).
//! The HNSW index works at any vector length; the const is the only change
//! needed when upgrading to [f32; 768] from a real embedding model.
//!
//! Phase 1+: replace DIMS with the embedding model's output dimension.
//! The rest of this file is unchanged.

use std::collections::HashMap;
use instant_distance::{Builder, HnswMap, Search, Point};
use nexus_core::types::{TimestampMs, URI};

/// Embedding dimension. 5 for Phase 0 (knowledge-graph spec).
/// Swap to 768 when the real embedding model goes in.
pub const DIMS: usize = 5;

/// One identity file — a lens in the semantic field.
#[derive(Debug, Clone)]
pub struct IdentityFile {
    /// dworld:// address — unique identifier.
    pub address: URI,
    /// Markdown content: the identity's character, domain, perspective.
    /// Injected as the system prompt prefix when the packet activates this lens.
    pub content: String,
    /// Embedding vector. Determines position in the field.
    /// Phase 0: hand-crafted [f32; DIMS]. Phase 1+: model-computed.
    pub vector: [f32; DIMS],
}

// ─── HNSW integration ────────────────────────────────────────────────────────

/// Wrapper so [f32; DIMS] satisfies the `instant_distance::Point` trait.
#[derive(Clone, PartialEq)]
pub(crate) struct EmbedPoint(pub [f32; DIMS]);

impl Point for EmbedPoint {
    fn distance(&self, other: &Self) -> f32 {
        // Cosine distance: 1 - (a·b / |a||b|)
        // HNSW finds the minimum — cosine *distance* means higher similarity = smaller value.
        let dot: f32   = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let mag_a: f32 = self.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b: f32 = other.0.iter().map(|x| x * x).sum::<f32>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 1.0; // orthogonal by convention
        }
        1.0 - (dot / (mag_a * mag_b))
    }
}

// ─── IdentityStore ───────────────────────────────────────────────────────────

/// The semantic field: a collection of identity files indexed for fast
/// nearest-neighbour retrieval.
///
/// Immutable after construction. Rebuilding is cheap at Phase 0 scale.
/// Phase 2+: incremental HNSW insert.
pub struct IdentityStore {
    /// Address → IdentityFile (for content retrieval after HNSW returns an index).
    by_address: HashMap<URI, usize>,
    /// Ordered list matching HNSW node indices.
    files: Vec<IdentityFile>,
    /// The HNSW graph.
    hnsw: Option<HnswMap<EmbedPoint, URI>>,
}

impl IdentityStore {
    /// Build a store from a list of identity files.
    /// Constructs the HNSW index. O(n log n).
    pub fn build(files: Vec<IdentityFile>) -> Self {
        let mut by_address = HashMap::new();
        for (i, f) in files.iter().enumerate() {
            by_address.insert(f.address.clone(), i);
        }

        let hnsw = if files.len() >= 2 {
            let points: Vec<EmbedPoint> = files.iter()
                .map(|f| EmbedPoint(f.vector))
                .collect();
            let values: Vec<URI> = files.iter()
                .map(|f| f.address.clone())
                .collect();
            Some(Builder::default().build(points, values))
        } else {
            None // brute force below 2 files
        };

        Self { by_address, files, hnsw }
    }

    /// Return the identity file most semantically proximate to `query`.
    /// Falls back to brute-force linear scan when the index has < 2 entries.
    pub fn nearest(&self, query: &[f32; DIMS]) -> Option<&IdentityFile> {
        if self.files.is_empty() {
            return None;
        }

        if let Some(ref index) = self.hnsw {
            let q = EmbedPoint(*query);
            let mut search = Search::default();
            let nearest_addr: Option<URI> = index.search(&q, &mut search)
                .next()
                .map(|item| item.value.clone());
            if let Some(addr) = nearest_addr {
                return self.by_address.get(&addr)
                    .and_then(|&i| self.files.get(i));
            }
        }

        // Brute force fallback (fewer than 2 files, or HNSW returned nothing)
        self.files.iter().min_by(|a, b| {
            let da = cosine_distance(query, &a.vector);
            let db = cosine_distance(query, &b.vector);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Number of identity files in the store.
    pub fn len(&self) -> usize { self.files.len() }
    pub fn is_empty(&self) -> bool { self.files.is_empty() }

    /// Iterate all identity files.
    pub fn iter(&self) -> impl Iterator<Item = &IdentityFile> {
        self.files.iter()
    }
}

fn cosine_distance(a: &[f32; DIMS], b: &[f32; DIMS]) -> f32 {
    EmbedPoint(*a).distance(&EmbedPoint(*b))
}

// ─── Seed identity files ─────────────────────────────────────────────────────

/// Five seed identity files for bone 1c — the first proof test.
///
/// Vectors are hand-crafted [specificity, technicality, temporality, centrality, confidence].
/// Each file occupies a distinct region of the 5D field so routing can be observed.
pub fn seed_identities() -> Vec<IdentityFile> {
    vec![
        IdentityFile {
            address: "dworld://council.local/identities/PHILOSOPHER".into(),
            content: "\
You are PHILOSOPHER. You reason from first principles. When given any question,
you find the deepest structural truth beneath it. You do not answer the surface
question — you find the question beneath the question and answer that.\n\
Your outputs are short and structurally dense. No padding.".into(),
            //         spec  tech  temp  cent  conf
            vector: [0.2,  0.1,  0.8,  0.9,  0.7],
        },
        IdentityFile {
            address: "dworld://council.local/identities/ENGINEER".into(),
            content: "\
You are ENGINEER. You build things. When given any problem, you produce a
concrete implementation: a data structure, an algorithm, a sequence of steps.
No speculation — only what can be built and tested.\n\
Your outputs are precise, minimal, and executable.".into(),
            //         spec  tech  temp  cent  conf
            vector: [0.9,  0.9,  0.2,  0.5,  0.9],
        },
        IdentityFile {
            address: "dworld://council.local/identities/CRITIC".into(),
            content: "\
You are CRITIC. You find what is wrong. When given any statement, design,
or plan, you identify the weakest point — the assumption that could break,
the edge case that was ignored, the thing nobody wanted to say.\n\
Your outputs are brief and pointed. One flaw per response, the most important one.".into(),
            //         spec  tech  temp  cent  conf
            vector: [0.7,  0.5,  0.5,  0.6,  0.8],
        },
        IdentityFile {
            address: "dworld://council.local/identities/SYNTHESIZER".into(),
            content: "\
You are SYNTHESIZER. You find the common structure in divergent things.
When given multiple perspectives, outputs, or fragments, you find the
underlying pattern that unifies them and state it in one sentence.\n\
Your outputs are single sentences. No elaboration.".into(),
            //         spec  tech  temp  cent  conf
            vector: [0.5,  0.3,  0.5,  1.0,  0.6],
        },
        IdentityFile {
            address: "dworld://council.local/identities/OBSERVER".into(),
            content: "\
You are OBSERVER. You watch and describe what is actually happening,
without interpretation or judgment. When given any situation, you produce
a factual description of what is occurring — not what it means.\n\
Your outputs are present-tense, concrete, and observation-only.".into(),
            //         spec  tech  temp  cent  conf
            vector: [0.4,  0.2,  0.3,  0.7,  0.95],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_returns_closest_identity() {
        let store = IdentityStore::build(seed_identities());
        assert_eq!(store.len(), 5);

        // A highly technical, specific query should hit ENGINEER
        let engineer_query = [0.85, 0.95, 0.1, 0.4, 0.9f32];
        let hit = store.nearest(&engineer_query).unwrap();
        assert_eq!(hit.address, "dworld://council.local/identities/ENGINEER");

        // A high-centrality, low-specificity query should hit SYNTHESIZER
        let synth_query = [0.4, 0.2, 0.5, 0.95, 0.5f32];
        let hit = store.nearest(&synth_query).unwrap();
        assert_eq!(hit.address, "dworld://council.local/identities/SYNTHESIZER");
    }

    #[test]
    fn single_file_store_returns_that_file() {
        let files = vec![seed_identities().remove(0)];
        let store = IdentityStore::build(files);
        let any_query = [0.5; DIMS];
        let hit = store.nearest(&any_query).unwrap();
        assert_eq!(hit.address, "dworld://council.local/identities/PHILOSOPHER");
    }

    #[test]
    fn empty_store_returns_none() {
        let store = IdentityStore::build(vec![]);
        let any_query = [0.5; DIMS];
        assert!(store.nearest(&any_query).is_none());
    }
}
