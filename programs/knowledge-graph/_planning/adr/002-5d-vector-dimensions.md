# ADR-002: 5D Vector — Dimension Definitions

Status: accepted
Date: 2026-03-13
Source: User specified "5D vector" — dimension semantics inferred and accepted here

## Decision
Five dimensions: Specificity, Technicality, Temporality, Centrality, Confidence.
All floats in [0.0, 1.0]. See `_planning/5d-vector-spec.md` for full definitions.

## Rationale
- User specified 5 dimensions explicitly
- 5 dimensions chosen to be orthogonal (low correlation between dimensions)
- Each dimension interpretable by a human reading the raw values
- Dimensions cover the primary axes of variation for knowledge documents:
  - WHAT kind of content (Specificity, Technicality)
  - WHEN it applies (Temporality)
  - WHERE it sits in the graph (Centrality)
  - HOW certain it is (Confidence)

## Consequences
- indexer must compute all 5 values for every file
- Centrality (dim 4) is the only dimension that improves over time (from ticker data)
  — initial estimates will be inaccurate; accept this as a known limitation
- ML upgrade (projecting from high-D embeddings) is possible without format change
- Proximity between documents is meaningful but imperfect — 5D captures only
  the five chosen axes; domain similarity within a cluster is not captured
- Two documents can have identical 5D positions but different content — this is
  expected and handled by the neighbor list + context files
