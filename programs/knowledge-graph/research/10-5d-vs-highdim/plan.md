# Study 10 — 5D vs High-Dimensional Embeddings

Status: designed
Priority: LOW — long-game architectural question

---

## Research Question

Are 5 interpretable dimensions as useful as 1536-dimensional embeddings (OpenAI ada-002)
or 4096-dimensional embeddings (Claude) for the specific task of document navigation?

This study determines whether the ML upgrade path is worth pursuing.

---

## Hypothesis

**H1:** For navigation quality (reaching a goal in fewer hops), 5D heuristic vectors
are within 20% of high-dimensional embeddings on the 20-file test set.

**H2:** For neighbor relevance, high-dimensional embeddings will be more accurate
because they capture topic similarity at a granularity 5D cannot.

**H3:** The gap between 5D and 1536D is smaller than expected because:
- Document navigation benefits from interpretability (AI can reason about "this is a hub")
- 5D positions can be explained by the AI; 1536D cannot

---

## When to Run This Study

Prerequisites:
1. Study 02 (heuristic accuracy) is concluded — we need a calibrated heuristic
2. Study 03 (neighbor relevance) is concluded — we have a baseline
3. The system has been running for at least 2 weeks with 50+ files

---

## Test Protocol

### Setup
Use the same 50+ file corpus (real documents from this workspace, not test documents).

**Condition A:** Heuristic 5D vectors (from Study 02's calibrated implementation)
**Condition B:** OpenAI text-embedding-3-small vectors → projected to 5D via PCA
**Condition C:** Full OpenAI text-embedding-3-small vectors (1536D) — baseline ceiling

### Navigation test (same goal, 3 runs each)
Goal: "Find all documents about architectural patterns and conventions."

Measure:
- Reads to goal
- % relevant reads
- Jaccard similarity between found set and expected set

### Neighbor accuracy test
For each file, compare top-5 neighbors under each condition.
Jaccard similarity between condition A neighbors and condition C neighbors.

---

## Decision Points

If A vs C gap < 15% on navigation quality:
→ Keep 5D heuristics. ML upgrade not needed.

If A vs C gap 15-30%:
→ Investigate Condition B (PCA projection). Does it close the gap?
→ If yes: pursue ML projection. If no: keep heuristics.

If A vs C gap > 30%:
→ ML embeddings are required for good navigation.
→ Build the ML upgrade path. Heuristics become the bootstrap only.

---

## The Interpretability Trade-off

Even if ML embeddings are more accurate, there is a strong case for 5D:

The AI can read `vector: [0.8, 0.3, 0.1, 0.9, 0.9]` and say:
"This is a specific, accessible, foundational hub document."

It cannot reason from `vector: [0.021, -0.134, 0.892, ..., 0.003]`.

Document the interpretability benefit explicitly — even if ML wins on metrics,
the 5D system may be better for an agent-operated knowledge graph where the
AI is making navigation decisions based on what it can read and reason about.

This study may conclude: "Use ML embeddings for neighbor search but keep 5D
as the human-readable position label." That would be the hybrid architecture.
