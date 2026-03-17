# Study 08 — Scalability

Status: designed
Priority: MEDIUM — run when system has 100+ files

---

## Research Question

At what file count does the system start to degrade?
Which component degrades first?

---

## Hypothesis

**H1:** The indexer (k-NN search) degrades at O(n²) — comparing each file to all others.
At 1000 files: 1,000,000 comparisons per indexing run. Likely too slow without optimization.

**H2:** The file-selector remains fast at any scale (reading one file is O(1)).

**H3:** The proximity query (5D search) degrades linearly — manageable to ~10,000 files.

**H4:** ticker.log file size becomes a concern at 10,000+ reads but not earlier.

---

## Test Protocol

Benchmarks at three scales:
- **Small**: 10 files
- **Medium**: 100 files
- **Large**: 1000 files

For each scale, measure:

| Operation | 10 files | 100 files | 1000 files | Scaling |
|-----------|---------|---------|----------|---------|
| Index all files | — | — | — | O(?) |
| k-NN search for one file | — | — | — | O(?) |
| Proximity query | — | — | — | O(?) |
| context-builder one file | — | — | — | O(?) |
| ticker.log read (full) | — | — | — | O(?) |

### Generating test files at scale
Write a script: `research/08-scalability/scripts/generate_test_files.py`
Generates N files with valid format but placeholder content.
Content is varied enough to produce different vector values.

---

## Optimization Triggers

If indexer is too slow at 1000 files:
- Option A: Index only on creation + on significant content change (not on every run)
- Option B: Use approximate k-NN (skip files outside a broad proximity threshold)
- Option C: Pre-build a spatial index (k-d tree in 5D)

Document which optimization is needed and add it as a gap in _meta/gaps/pending.txt.

---

## Success Criteria

System remains "usable" (indexing completes in < 30 seconds) at 1000 files.
If not: the optimization path is documented and becomes a build task.
