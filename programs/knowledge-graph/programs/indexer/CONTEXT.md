# programs/indexer — Task Router

## What This Program Is
Computes 5D position vectors for every file. Finds k nearest neighbors.
Writes results back into file metadata. Runs on new files and re-indexes on change.
Heuristic method first. ML projection is the upgrade path (no format change).

---

## Before Writing Any Code
Read ADR-002 (dimension definitions) and ADR-003 (heuristic method).
Read ADR-004 (k=5 default).
Read shared/contracts/file-record.md — specifically the vector and neighbors fields.
Read _planning/5d-vector-spec.md §Computing Vectors for exact heuristic rules.
data-store must exist (files to index must exist).

---

## Task Routing

| Your Task | Load These | Skip These |
|-----------|-----------|------------|
| Implement Specificity heuristic | _planning/5d-vector-spec.md §Dimension 1 | other programs |
| Implement Technicality heuristic | _planning/5d-vector-spec.md §Dimension 2 | — |
| Implement Temporality heuristic | _planning/5d-vector-spec.md §Dimension 3 | — |
| Implement Centrality | _planning/5d-vector-spec.md §Dimension 4 + ticker.log | — |
| Implement Confidence | _planning/5d-vector-spec.md §Dimension 5 | — |
| Implement k-NN search | _planning/5d-vector-spec.md §Distance | — |
| Write vector back to file | shared/contracts/file-record.md §metadata | — |

---

## Heuristic Summary

| Dim | Signal | Low → High |
|-----|--------|-----------|
| Specificity | proper nouns, numbers, code blocks | generic terms → specific names |
| Technicality | technical vocabulary, symbols | prose → code |
| Temporality | words like "currently", dates in content | "always" → "today" |
| Centrality | outbound links in content + ticker co-access count | isolated → hub |
| Confidence | modal verbs ("might", "is", "confirmed") | "perhaps" → "verified" |

---

## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| One dimension implemented | Vector for 3 test files, spot-check values | approve / calibrate |
| All 5 dimensions done | Vectors for 5 files, neighbor lists | approve / re-weight |

## Audit
- [ ] All 5 vector values are floats in [0.0, 1.0]
- [ ] Neighbors list contains only file numbers that exist in Data/
- [ ] `last_indexed` timestamp is updated
- [ ] Re-indexing a file produces the same vector (deterministic)
- [ ] k-NN returns k files sorted by ascending distance

---

## Inputs
- `Data/file[NNNN].md` — file to index (reads content + metadata)
- `Data/ticker.log` — for centrality computation (dim 4)

## Outputs
- Updated `vector` field in file metadata
- Updated `neighbors` field in file metadata
- Updated `last_indexed` field in file metadata
