//! Identity file store with approximate nearest-neighbour retrieval.
//!
//! An identity file is the lens. The store is the ether — the collection
//! of all lenses suspended in the semantic field.
//!
//! Phase 0: vectors are hand-crafted [f32; 5] slices stored as Vec<f32>.
//! Phase 1+: vectors are 384-dim output from AllMiniLML6V2 (fastembed).
//! The HNSW index works at any vector length — only the embedding call changes.

use std::collections::HashMap;
use instant_distance::{Builder, HnswMap, Search, Point};
use nexus_core::types::URI;

/// One identity file — a lens in the semantic field.
#[derive(Debug, Clone)]
pub struct IdentityFile {
    /// dworld:// address — unique identifier.
    pub address: URI,
    /// Markdown content: the identity's character, domain, perspective.
    /// Injected as the system prompt prefix when the packet activates this lens.
    pub content: String,
    /// Embedding vector. Determines position in the field.
    /// Phase 0: hand-crafted 5D. Phase 1+: 384D from AllMiniLML6V2.
    pub vector: Vec<f32>,
    /// 3D coordinate in the physical world (the wire between semantic and physics layers).
    ///
    /// Set once at identity file creation from the force-directed layout algorithm
    /// (bone 3c). None until layout runs — bone 1/2 operate with world_coord = None.
    ///
    /// When a packet hops to this identity, it inherits this coordinate:
    /// packet.world_position = identity.world_coord. The packet's path through
    /// the semantic field becomes a physical trail through the 3D world.
    ///
    /// Changed only by the reformation system (bone 4) after content rewrite + re-embed
    /// + layout update. The lens does not have velocity — it has reformation.
    pub world_coord: Option<[f32; 3]>,
}

// ─── HNSW integration ────────────────────────────────────────────────────────

/// Wrapper so `Vec<f32>` satisfies the `instant_distance::Point` trait.
#[derive(Clone, PartialEq)]
pub(crate) struct EmbedPoint(pub Vec<f32>);

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
                .map(|f| EmbedPoint(f.vector.clone()))
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
    pub fn nearest(&self, query: &[f32]) -> Option<&IdentityFile> {
        if self.files.is_empty() {
            return None;
        }

        if let Some(ref index) = self.hnsw {
            let q = EmbedPoint(query.to_vec());
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

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    EmbedPoint(a.to_vec()).distance(&EmbedPoint(b.to_vec()))
}

// ─── Seed identity files ─────────────────────────────────────────────────────

/// Five seed identity files for bone 1c — the first proof test.
///
/// Vectors are hand-crafted 5D [specificity, technicality, temporality, centrality, confidence].
/// Each file occupies a distinct region of the field so routing can be observed.
/// Phase 1+: replace with real AllMiniLML6V2 embeddings of the content strings.
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
            vector: vec![0.2,  0.1,  0.8,  0.9,  0.7],
            world_coord: None, // assigned by bone 3c layout algorithm
        },
        IdentityFile {
            address: "dworld://council.local/identities/ENGINEER".into(),
            content: "\
You are ENGINEER. You build things. When given any problem, you produce a
concrete implementation: a data structure, an algorithm, a sequence of steps.
No speculation — only what can be built and tested.\n\
Your outputs are precise, minimal, and executable.".into(),
            //         spec  tech  temp  cent  conf
            vector: vec![0.9,  0.9,  0.2,  0.5,  0.9],
            world_coord: None,
        },
        IdentityFile {
            address: "dworld://council.local/identities/CRITIC".into(),
            content: "\
You are CRITIC. You find what is wrong. When given any statement, design,
or plan, you identify the weakest point — the assumption that could break,
the edge case that was ignored, the thing nobody wanted to say.\n\
Your outputs are brief and pointed. One flaw per response, the most important one.".into(),
            //         spec  tech  temp  cent  conf
            vector: vec![0.7,  0.5,  0.5,  0.6,  0.8],
            world_coord: None,
        },
        IdentityFile {
            address: "dworld://council.local/identities/SYNTHESIZER".into(),
            content: "\
You are SYNTHESIZER. You find the common structure in divergent things.
When given multiple perspectives, outputs, or fragments, you find the
underlying pattern that unifies them and state it in one sentence.\n\
Your outputs are single sentences. No elaboration.".into(),
            //         spec  tech  temp  cent  conf
            vector: vec![0.5,  0.3,  0.5,  1.0,  0.6],
            world_coord: None,
        },
        IdentityFile {
            address: "dworld://council.local/identities/OBSERVER".into(),
            content: "\
You are OBSERVER. You watch and describe what is actually happening,
without interpretation or judgment. When given any situation, you produce
a factual description of what is occurring — not what it means.\n\
Your outputs are present-tense, concrete, and observation-only.".into(),
            //         spec  tech  temp  cent  conf
            vector: vec![0.4,  0.2,  0.3,  0.7,  0.95],
            world_coord: None,
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
        let engineer_query = [0.85f32, 0.95, 0.1, 0.4, 0.9];
        let hit = store.nearest(&engineer_query).unwrap();
        assert_eq!(hit.address, "dworld://council.local/identities/ENGINEER");

        // A high-centrality, low-specificity query should hit SYNTHESIZER
        let synth_query = [0.4f32, 0.2, 0.5, 0.95, 0.5];
        let hit = store.nearest(&synth_query).unwrap();
        assert_eq!(hit.address, "dworld://council.local/identities/SYNTHESIZER");
    }

    #[test]
    fn single_file_store_returns_that_file() {
        let files = vec![seed_identities().remove(0)];
        let store = IdentityStore::build(files);
        let any_query = [0.5f32; 5];
        let hit = store.nearest(&any_query).unwrap();
        assert_eq!(hit.address, "dworld://council.local/identities/PHILOSOPHER");
    }

    #[test]
    fn empty_store_returns_none() {
        let store = IdentityStore::build(vec![]);
        let any_query = [0.5f32; 5];
        assert!(store.nearest(&any_query).is_none());
    }
}
