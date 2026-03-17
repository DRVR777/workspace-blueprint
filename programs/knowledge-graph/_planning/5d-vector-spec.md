# 5D Vector Spec — Semantic Coordinates

## Why 5 Dimensions

Standard embeddings (OpenAI: 1536D, Claude: 4096D) are not human-readable.
You cannot look at a 1536-element vector and understand what it means.

This system uses 5 **interpretable** dimensions. Each dimension has a clear semantic meaning.
A human (or agent) can read `[0.8, 0.2, 0.1, 0.9, 0.9]` and understand immediately:
"specific, intuitive, foundational, central hub, highly certain."

Trade-off: less precision than high-D embeddings.
Gain: interpretability, debuggability, human oversight, no ML required to bootstrap.

The indexer can be upgraded later to compute these dimensions from high-D embeddings
using projection. The format stays identical — only the computation changes.

---

## The 5 Dimensions

### Dimension 1 — Specificity
**What it measures:** How concrete and detailed is this document?

| Value | Meaning | Example |
|-------|---------|---------|
| 0.0–0.2 | Highly abstract | "Systems thinking principles" |
| 0.3–0.4 | Conceptual with some grounding | "Architecture decision framework" |
| 0.5 | Mixed | "REST API design patterns" |
| 0.6–0.7 | Mostly specific | "Authentication module spec" |
| 0.8–1.0 | Highly specific | "POST /auth/login endpoint contract" |

### Dimension 2 — Technicality
**What it measures:** Is this conceptual/intuitive or technical/formal?

| Value | Meaning | Example |
|-------|---------|---------|
| 0.0–0.2 | Intuitive, no technical knowledge required | Vision statement |
| 0.3–0.4 | Light technical framing | Product requirements |
| 0.5 | Mixed | System design doc |
| 0.6–0.7 | Technical audience required | API spec |
| 0.8–1.0 | Formal/code-level | Source code, schema definitions |

### Dimension 3 — Temporality
**What it measures:** How time-sensitive is this document?

| Value | Meaning | Example |
|-------|---------|---------|
| 0.0–0.2 | Foundational, will not change | Architectural principles |
| 0.3–0.4 | Stable but revisable | Accepted ADRs |
| 0.5 | Moderate lifespan | Feature specs |
| 0.6–0.7 | Expected to change | Work-in-progress notes |
| 0.8–1.0 | Current/ephemeral | Meeting notes, status updates |

### Dimension 4 — Centrality
**What it measures:** How connected is this document to others?

| Value | Meaning | Example |
|-------|---------|---------|
| 0.0–0.2 | Isolated, few connections | An appendix |
| 0.3–0.4 | Some connections | A module's implementation detail |
| 0.5 | Moderately connected | A program CONTEXT.md |
| 0.6–0.7 | Well connected | A shared contract |
| 0.8–1.0 | Hub node | Root CLAUDE.md, CONVENTIONS.md |

Note: Centrality starts as an estimate. It becomes accurate over time as the
ticker.log builds up actual connection data. The indexer updates centrality
based on ticker graph analysis.

### Dimension 5 — Confidence
**What it measures:** How certain/established is the content?

| Value | Meaning | Example |
|-------|---------|---------|
| 0.0–0.2 | Highly speculative | Brainstorm, hypothesis |
| 0.3–0.4 | Probable | Proposed ADR |
| 0.5 | Working assumption | Assumption-status ADR |
| 0.6–0.7 | Likely correct | Accepted ADR from PRD |
| 0.8–1.0 | Established fact | Implemented and verified system |

---

## Computing Vectors

### Method 1 — Heuristic (default for bootstrapping)

The indexer uses rules to assign values:

**Specificity:**
- Count nouns that are proper (names, specific things) vs common (categories, types)
- Ratio of concrete examples to abstract statements
- Presence of numbers, measurements, code = higher specificity

**Technicality:**
- Vocabulary analysis: presence of technical terms, symbols, code blocks
- Section headers: "Implementation" / "Code" = higher; "Vision" / "Goals" = lower

**Temporality:**
- Metadata: created date, last_modified
- Language signals: "currently", "today", "sprint" = higher temporal
- "always", "principle", "foundation" = lower temporal

**Centrality:**
- Count of links/references to other files in content
- Updated from ticker.log: files with many co-access events = higher centrality

**Confidence:**
- Language signals: "we know", "verified", "tested" = higher
- "hypothesis", "might", "assume" = lower
- ADR status: `accepted` → 0.8+; `assumption` → 0.4; `proposed` → 0.3

### Method 2 — ML projection (future upgrade)

1. Embed document using high-D model
2. Project to 5D using PCA or trained projection matrix
3. Normalize each dimension to [0.0, 1.0]
4. Override: map PCA axes to semantic dimensions via labeled examples

This upgrade requires no format change — only the indexer changes internally.

---

## Distance and Neighbors

Proximity = Euclidean distance in 5D space:

```
distance(A, B) = sqrt(
    (A1-B1)² + (A2-B2)² + (A3-B3)² + (A4-B4)² + (A5-B5)²
)
```

Maximum possible distance = sqrt(5) ≈ 2.236 (opposite corners of unit hypercube).

k-nearest neighbors = the k files with smallest distance.
Default k = 5. Configurable per-file by setting a `k: N` field in metadata.

**Neighbor semantics:**
Two files with distance < 0.5 are strongly related.
Distance 0.5–1.0: moderately related.
Distance > 1.0: likely different clusters.

---

## Reading a Vector

When an AI reads `vector: [0.8, 0.3, 0.2, 0.9, 0.9]`:

```
Specificity:  0.8 → Highly specific
Technicality: 0.3 → Light technical framing
Temporality:  0.2 → Stable/foundational
Centrality:   0.9 → Hub node
Confidence:   0.9 → Established/verified
```

Interpretation: "This is a specific, accessible, foundational hub document that is
well-established. Likely a key reference doc or architectural specification."

---

## Dimension Weights (for proximity queries)

When the AI queries by 5D position to find related content, it can weight dimensions:

```python
# Find documents similar to a hub ADR
query_vector = [0.7, 0.5, 0.2, 0.8, 0.85]
weights = [1.0, 0.5, 2.0, 0.5, 1.5]  # emphasize temporality and confidence
```

Weighted distance:
```
distance = sqrt(sum(w_i * (A_i - B_i)² for i in 1..5))
```

Default weights: [1.0, 1.0, 1.0, 1.0, 1.0] (equal).
Weights are passed as parameters to file-selector, not stored in files.
