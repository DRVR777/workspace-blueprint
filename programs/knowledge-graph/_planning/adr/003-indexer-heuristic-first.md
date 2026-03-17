# ADR-003: Indexer — Heuristic Computation First, ML Upgrade Later

Status: accepted
Date: 2026-03-13
Source: Inferred — user did not specify computation method

## Decision
The indexer uses heuristic rules (vocabulary analysis, language signals, metadata)
to compute 5D vectors. No ML model required for the initial build.
The ML projection path is documented but not implemented.

## Rationale
- Heuristic approach runs without an API call, without a model, without any infra
- Results in the correct range immediately — good enough to find meaningful neighbors
- The format is identical whether computed by heuristic or ML — upgrade path is clean
- Bootstrapping is fast: can index 100 files without waiting for embeddings
- For this workspace (markdown files, known document types), heuristics are accurate
  enough to provide useful proximity

## Consequences
- Vectors will be less precise than ML-computed embeddings
- Two stylistically similar but semantically different documents may be close neighbors
  — context files will surface this and the AI can navigate past it
- Centrality dimension starts as a rough estimate — becomes accurate after 10+ sessions
- The heuristic method is documented in 5d-vector-spec.md so any agent can re-implement it
- When upgrading to ML: only indexer/src/ changes. No format changes. No migration needed.
