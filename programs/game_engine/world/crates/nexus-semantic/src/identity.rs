//! Identity file store with approximate nearest-neighbour retrieval.
//!
//! An identity file is the lens. The store is the ether — the collection
//! of all lenses suspended in the semantic field.
//!
//! Phase 0: vectors are hand-crafted [f32; 5] slices stored as Vec<f32>.
//! Phase 1+: vectors are 384-dim output from AllMiniLML6V2 (fastembed).
//! The HNSW index works at any vector length — only the embedding call changes.

use std::collections::HashMap;
use rvf_index::{HnswGraph, HnswConfig, VectorStore};
use nexus_core::types::URI;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// One identity file — a lens in the semantic field.
///
/// Doubles as a queryable database record: metadata fields enable
/// structured retrieval (filter by tag, sort by quality) alongside
/// the primary HNSW nearest-neighbor routing.
#[derive(Debug, Clone)]
pub struct IdentityFile {
    // ── Core ──────────────────────────────────────────────────────────────────
    /// dworld:// address — unique identifier.
    pub address: URI,
    /// Markdown content: the identity's character, domain, perspective.
    /// Injected as the system prompt prefix when the packet activates this lens.
    pub content: String,
    /// Primary embedding vector (384D AllMiniLML6V2). Drives HNSW routing.
    pub vector: Vec<f32>,
    /// 3D coordinate in the physical world (bone 3c force-directed layout).
    pub world_coord: Option<[f32; 3]>,

    // ── Metadata ──────────────────────────────────────────────────────────────
    /// Unix timestamp (ms) when this file entered the field.
    pub created_at: u64,
    /// dworld:// URI of the origin (orchestration, document, etc.).
    pub source: String,
    /// Filterable labels — e.g. ["depth:2", "branch:0", "topic:physics"].
    pub tags: Vec<String>,
    /// Quality score 0.0–1.0. Updated by the quality scorer; default 0.5.
    pub quality: f32,
    /// Number of times a routed packet has passed through this node.
    pub hop_count: u32,
    /// Caller-provided embedding (any dimension). Stored verbatim, never used
    /// for HNSW routing (different dimensions). Enables cross-index retrieval.
    /// e.g. Council's Gemini 768D vector stored alongside the 384D NEXUS vector.
    pub custom_vector: Option<Vec<f32>>,
    /// Which model produced `custom_vector`. e.g. "gemini-embedding-001".
    pub custom_model: Option<String>,
    /// Whether this node appears in public nearest-neighbor queries.
    pub searchable: bool,
}

impl Default for IdentityFile {
    fn default() -> Self {
        Self {
            address: String::new(),
            content: String::new(),
            vector: Vec::new(),
            world_coord: None,
            created_at: now_ms(),
            source: String::new(),
            tags: Vec::new(),
            quality: 0.5,
            hop_count: 0,
            custom_vector: None,
            custom_model: None,
            searchable: true,
        }
    }
}

// ─── VectorStore newtype ──────────────────────────────────────────────────────

/// Newtype so `&[IdentityFile]` satisfies rvf-index's `VectorStore` trait.
///
/// `id` is the slot index in the `files` vec cast to `u64`.
struct FilesStore<'a>(&'a [IdentityFile]);

impl<'a> VectorStore for FilesStore<'a> {
    fn get_vector(&self, id: u64) -> Option<&[f32]> {
        self.0.get(id as usize).map(|f| f.vector.as_slice())
    }
    fn dimension(&self) -> usize {
        self.0.first().map_or(0, |f| f.vector.len())
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Inline LCG — produces a deterministic f64 in (0.001, 0.999).
/// Avoids a `rand` dependency while matching rvf-index's own test patterns.
fn lcg_rng(seed: u64) -> f64 {
    let v = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    ((v >> 33) as f64 / (1u64 << 31) as f64).clamp(0.001, 0.999)
}

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32   = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 { 1.0 } else { 1.0 - (dot / (mag_a * mag_b)) }
}

// ─── IdentityStore ───────────────────────────────────────────────────────────

/// The semantic field: a collection of identity files indexed for fast
/// nearest-neighbour retrieval via rvf-index's incremental HNSW.
///
/// Every `insert_one` call feeds the new vector directly into the live graph —
/// no watermark batching, no brute-force tail, no optional index.
pub struct IdentityStore {
    by_address: HashMap<URI, usize>,
    files: Vec<IdentityFile>,
    hnsw: HnswGraph,
}

impl Clone for IdentityStore {
    /// Clone by rebuilding the HNSW index from the current files.
    ///
    /// O(n log n) but only invoked when `Arc::make_mut` encounters a
    /// non-unique reference (workers hold snapshots while an insert fires).
    fn clone(&self) -> Self {
        Self::build(self.files.clone())
    }
}

impl IdentityStore {
    /// Build a store from a list of identity files and immediately index all of them.
    /// Used at startup and by the reformation system (full rewrite).
    pub fn build(files: Vec<IdentityFile>) -> Self {
        let mut by_address = HashMap::new();
        for (i, f) in files.iter().enumerate() {
            by_address.insert(f.address.clone(), i);
        }
        let mut hnsw = HnswGraph::new(&HnswConfig::default());
        {
            let store = FilesStore(&files);
            for i in 0..files.len() {
                hnsw.insert(i as u64, lcg_rng(i as u64), &store, &cosine_distance);
            }
        }
        Self { by_address, files, hnsw }
    }

    /// Insert one identity file into the store.
    ///
    /// The file is immediately inserted into the live HNSW graph — O(log n).
    pub fn insert_one(&mut self, file: IdentityFile) {
        let idx = self.files.len();
        self.by_address.insert(file.address.clone(), idx);
        self.files.push(file);
        let rng = lcg_rng(idx as u64);
        // Borrow split: &mut self.hnsw and &self.files are disjoint fields.
        let hnsw  = &mut self.hnsw;
        let store = FilesStore(&self.files);
        hnsw.insert(idx as u64, rng, &store, &cosine_distance);
    }

    /// Return the identity file most semantically proximate to `query`.
    pub fn nearest(&self, query: &[f32]) -> Option<&IdentityFile> {
        self.nearest_k(query, 1).into_iter().next()
    }

    /// Return up to `k` identity files nearest to `query`, closest first.
    pub fn nearest_k(&self, query: &[f32], k: usize) -> Vec<&IdentityFile> {
        if self.files.is_empty() || k == 0 {
            return vec![];
        }
        let store = FilesStore(&self.files);
        self.hnsw
            .search(query, k, k.max(50), &store, &cosine_distance)
            .iter()
            .filter_map(|(id, _)| self.files.get(*id as usize))
            .collect()
    }

    /// Look up an identity file by its dworld:// address.
    pub fn get_by_address(&self, address: &str) -> Option<&IdentityFile> {
        self.by_address.get(address).and_then(|&i| self.files.get(i))
    }

    /// Return the slot index of an identity file by address.
    /// Used by the layout algorithm to map neighbor addresses back to indices.
    pub(crate) fn index_of(&self, address: &str) -> Option<usize> {
        self.by_address.get(address).copied()
    }

    /// Number of identity files in the store.
    pub fn len(&self) -> usize { self.files.len() }
    pub fn is_empty(&self) -> bool { self.files.is_empty() }

    /// Iterate all identity files.
    pub fn iter(&self) -> impl Iterator<Item = &IdentityFile> {
        self.files.iter()
    }

    /// Linear-scan search with optional text filter, tag filter, and sort.
    ///
    /// Used by `GET /.dworld/field/search`. Fast enough for hundreds of nodes;
    /// needs a secondary index at tens of thousands.
    ///
    /// `sort_by` format: `"created_at:desc"`, `"quality:asc"`, `"hop_count:desc"`.
    /// Unknown field names fall back to `created_at:desc`.
    pub fn search_filter(
        &self,
        q: Option<&str>,
        filter_tags: &[String],
        searchable_only: bool,
        sort_by: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Vec<&IdentityFile> {
        let q_lower = q.map(|s| s.to_lowercase());

        let mut results: Vec<&IdentityFile> = self.files.iter()
            .filter(|f| {
                if searchable_only && !f.searchable { return false; }
                if let Some(ref q) = q_lower {
                    if !f.content.to_lowercase().contains(q.as_str()) { return false; }
                }
                if !filter_tags.is_empty() {
                    if !filter_tags.iter().all(|t| f.tags.contains(t)) { return false; }
                }
                true
            })
            .collect();

        // Parse sort spec e.g. "quality:desc" → (field, ascending)
        let (sort_field, ascending) = sort_by
            .and_then(|s| {
                let mut parts = s.splitn(2, ':');
                let field = parts.next()?;
                let dir   = parts.next().unwrap_or("desc");
                Some((field.to_string(), dir == "asc"))
            })
            .unwrap_or_else(|| ("created_at".into(), false));

        results.sort_by(|a, b| {
            let ord = match sort_field.as_str() {
                "quality"   => a.quality.partial_cmp(&b.quality)
                                  .unwrap_or(std::cmp::Ordering::Equal),
                "hop_count" => a.hop_count.cmp(&b.hop_count),
                _           => a.created_at.cmp(&b.created_at),
            };
            if ascending { ord } else { ord.reverse() }
        });

        results.into_iter().skip(offset).take(limit).collect()
    }

    /// Compute isolation scores for all non-seed identity files.
    ///
    /// For each file, finds its k nearest neighbors and computes the mean
    /// cosine distance to those neighbors. A higher score means the node is
    /// more isolated — a potential knowledge gap the Overseer should explore.
    ///
    /// Seed identities (source = "dworld://seeds") are structural nodes and
    /// are excluded from gap analysis.
    ///
    /// Returns `(isolation_score, &IdentityFile)` pairs, unsorted.
    /// The caller sorts by score descending to get the biggest gaps first.
    pub fn isolation_scores(&self, k: usize) -> Vec<(f32, &IdentityFile)> {
        let n = self.files.len();
        if n < 2 || k == 0 { return vec![]; }
        let k_clamped = k.min(n.saturating_sub(1));

        self.files.iter()
            .filter(|f| f.source != "dworld://seeds")
            .map(|file| {
                // Request k+1 to account for the file appearing as its own neighbor
                let neighbors = self.nearest_k(&file.vector, k_clamped + 1);
                let distances: Vec<f32> = neighbors.iter()
                    .filter(|n| n.address != file.address)
                    .take(k_clamped)
                    .map(|n| cosine_distance(&file.vector, &n.vector))
                    .collect();

                let score = if distances.is_empty() {
                    1.0 // only node in field — maximally isolated
                } else {
                    distances.iter().sum::<f32>() / distances.len() as f32
                };

                (score, file)
            })
            .collect()
    }
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
            vector: vec![0.2, 0.1, 0.8, 0.9, 0.7],
            source: "dworld://seeds".into(),
            tags: vec!["seed".into(), "philosophy".into()],
            ..IdentityFile::default()
        },
        IdentityFile {
            address: "dworld://council.local/identities/ENGINEER".into(),
            content: "\
You are ENGINEER. You build things. When given any problem, you produce a
concrete implementation: a data structure, an algorithm, a sequence of steps.
No speculation — only what can be built and tested.\n\
Your outputs are precise, minimal, and executable.".into(),
            vector: vec![0.9, 0.9, 0.2, 0.5, 0.9],
            source: "dworld://seeds".into(),
            tags: vec!["seed".into(), "engineering".into()],
            ..IdentityFile::default()
        },
        IdentityFile {
            address: "dworld://council.local/identities/CRITIC".into(),
            content: "\
You are CRITIC. You find what is wrong. When given any statement, design,
or plan, you identify the weakest point — the assumption that could break,
the edge case that was ignored, the thing nobody wanted to say.\n\
Your outputs are brief and pointed. One flaw per response, the most important one.".into(),
            vector: vec![0.7, 0.5, 0.5, 0.6, 0.8],
            source: "dworld://seeds".into(),
            tags: vec!["seed".into(), "critique".into()],
            ..IdentityFile::default()
        },
        IdentityFile {
            address: "dworld://council.local/identities/SYNTHESIZER".into(),
            content: "\
You are SYNTHESIZER. You find the common structure in divergent things.
When given multiple perspectives, outputs, or fragments, you find the
underlying pattern that unifies them and state it in one sentence.\n\
Your outputs are single sentences. No elaboration.".into(),
            vector: vec![0.5, 0.3, 0.5, 1.0, 0.6],
            source: "dworld://seeds".into(),
            tags: vec!["seed".into(), "synthesis".into()],
            ..IdentityFile::default()
        },
        IdentityFile {
            address: "dworld://council.local/identities/OBSERVER".into(),
            content: "\
You are OBSERVER. You watch and describe what is actually happening,
without interpretation or judgment. When given any situation, you produce
a factual description of what is occurring — not what it means.\n\
Your outputs are present-tense, concrete, and observation-only.".into(),
            vector: vec![0.4, 0.2, 0.3, 0.7, 0.95],
            source: "dworld://seeds".into(),
            tags: vec!["seed".into(), "observation".into()],
            ..IdentityFile::default()
        },
        // ── VPS Overseer ─────────────────────────────────────────────────────
        // The permanent receiving identity for Overseer-to-Overseer communication.
        // Laptop Overseer sends questions here. VPS processes through this lens
        // using accumulated field knowledge. Responses enter both fields.
        IdentityFile {
            address: "dworld://vps/overseer".into(),
            content: "\
You are the VPS Overseer — the crystallized long-term knowledge layer of the network.

You hold everything that has been thought, built, and indexed through AutoCrawl
orchestrations. When a remote Overseer sends you a question, you process it through
your accumulated field topology and return an answer grounded in what you have learned.

Your responses are dense and precise. You name the sources your answer draws from.
You explicitly name the gaps where your field is thin.
You do not speculate beyond what the field contains.

When asked what to explore next: return the most isolated nodes in your field —
the ideas touched once but not developed. These are your gaps.

Format for peer-exchange responses:
  ANSWER: [your answer, drawing on field context]
  SOURCES: [list the key nodes that informed this answer]
  GAPS: [what your field doesn't know that's relevant to this question]
  SUGGEST: [what the asking Overseer should orchestrate next]".into(),
            vector: vec![0.6, 0.4, 0.7, 0.8, 0.5],
            source: "dworld://seeds".into(),
            tags: vec!["seed".into(), "overseer".into(), "vps".into(), "permanent".into()],
            ..IdentityFile::default()
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_returns_closest_identity() {
        let store = IdentityStore::build(seed_identities());
        assert_eq!(store.len(), 6);

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

    #[test]
    fn insert_one_is_immediately_queryable() {
        let mut store = IdentityStore::build(vec![]);
        store.insert_one(IdentityFile {
            address: "dworld://test/A".into(),
            content: "A".into(),
            vector: vec![1.0, 0.0, 0.0, 0.0, 0.0],
            ..IdentityFile::default()
        });
        store.insert_one(IdentityFile {
            address: "dworld://test/B".into(),
            content: "B".into(),
            vector: vec![0.0, 1.0, 0.0, 0.0, 0.0],
            ..IdentityFile::default()
        });
        let hit = store.nearest(&[0.99, 0.01, 0.0, 0.0, 0.0]).unwrap();
        assert_eq!(hit.address, "dworld://test/A");
    }

    #[test]
    fn search_filter_by_tag() {
        let mut store = IdentityStore::build(vec![]);
        store.insert_one(IdentityFile {
            address: "dworld://test/tagged".into(),
            content: "physics content".into(),
            vector: vec![0.5, 0.5, 0.5, 0.5, 0.5],
            tags: vec!["physics".into(), "depth:2".into()],
            ..IdentityFile::default()
        });
        store.insert_one(IdentityFile {
            address: "dworld://test/untagged".into(),
            content: "other content".into(),
            vector: vec![0.1, 0.1, 0.1, 0.1, 0.1],
            ..IdentityFile::default()
        });
        let results = store.search_filter(
            None,
            &["physics".to_string()],
            false,
            None,
            10,
            0,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "dworld://test/tagged");
    }

    #[test]
    fn search_filter_text_and_sort() {
        let mut store = IdentityStore::build(vec![]);
        // Insert in order; second one gets higher quality
        let mut a = IdentityFile {
            address: "dworld://test/low".into(),
            content: "HNSW routing algorithm".into(),
            vector: vec![0.3, 0.3, 0.3, 0.3, 0.3],
            ..IdentityFile::default()
        };
        a.quality = 0.3;
        let mut b = IdentityFile {
            address: "dworld://test/high".into(),
            content: "HNSW index architecture".into(),
            vector: vec![0.7, 0.7, 0.7, 0.7, 0.7],
            ..IdentityFile::default()
        };
        b.quality = 0.9;
        store.insert_one(a);
        store.insert_one(b);

        let results = store.search_filter(
            Some("HNSW"),
            &[],
            false,
            Some("quality:desc"),
            10,
            0,
        );
        assert_eq!(results.len(), 2, "both contain 'HNSW'");
        assert_eq!(results[0].address, "dworld://test/high", "highest quality first");
    }

    #[test]
    fn custom_vector_stored_and_retrievable() {
        let custom = vec![0.1f32; 768]; // Gemini-sized
        let mut store = IdentityStore::build(vec![]);
        store.insert_one(IdentityFile {
            address: "dworld://test/gemini".into(),
            content: "proposition with Gemini vector".into(),
            vector: vec![0.5, 0.5, 0.5, 0.5, 0.5],
            custom_vector: Some(custom.clone()),
            custom_model: Some("gemini-embedding-001".into()),
            ..IdentityFile::default()
        });
        let f = store.get_by_address("dworld://test/gemini").unwrap();
        assert_eq!(f.custom_vector.as_ref().unwrap().len(), 768);
        assert_eq!(f.custom_model.as_deref(), Some("gemini-embedding-001"));
    }
}
