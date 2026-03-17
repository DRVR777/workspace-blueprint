# Study 03 — Neighbor Relevance: Raw Scores

Date: 2026-03-16
Session ID used: study-03
Files evaluated: 25 (0001–0025)
Neighbor pairs scored: 110 (0001 has no neighbors; all others have 5)

---

## Scoring Rubric

| Score | Meaning |
|-------|---------|
| 5 | Directly related — essential for understanding the source |
| 4 | Closely related — adds useful context |
| 3 | Tangentially related — might be useful |
| 2 | Weakly related — surface similarity, little content overlap |
| 1 | Unrelated — proximity in 5D space was misleading |

---

## File Inventory (vector, title)

| File | Vector | Title |
|------|--------|-------|
| 0001 | [0.30, 0.20, 0.25, 0.10, 0.50] | Synthetic test file #1 |
| 0002 | [0.355, 0.20, 0.25, 0.10, 0.57] | Synthetic test file #2 (named ADR-002) |
| 0003 | [0.345, 0.20, 0.675, 0.10, 0.57] | Meeting notes 2026-03-14 |
| 0004 | [0.35, 0.36, 0.30, 0.10, 0.50] | File-selector proximity query implementation |
| 0005 | [0.30, 0.20, 0.25, 0.10, 0.29] | Hypothesis about centrality |
| 0006 | [1.00, 0.66, 0.21, 0.50, 0.43] | Architecture overview (full system diagram) |
| 0007 | [0.90, 0.56, 0.025, 0.26, 0.43] | PRD source |
| 0008 | [0.94, 0.63, 0.36, 0.10, 0.57] | RAG vs CDS comparison |
| 0009 | [1.00, 0.65, 0.35, 0.10, 0.50] | 5D vector spec |
| 0010 | [1.00, 0.67, 0.12, 0.10, 0.57] | File format spec |
| 0011 | [0.45, 0.24, 0.295, 0.10, 0.72] | ADR-001: file format decision |
| 0012 | [0.51, 0.15, 0.245, 0.10, 0.72] | ADR-002: 5D dimensions decision |
| 0013 | [0.48, 0.24, 0.245, 0.10, 0.79] | ADR-003: heuristic-first decision |
| 0014 | [0.53, 0.20, 0.27, 0.10, 0.65] | ADR-004: k=5 decision |
| 0015 | [0.525, 0.20, 0.32, 0.18, 0.72] | ADR-005: on-read trigger decision |
| 0016 | [0.525, 0.52, 0.195, 0.10, 0.72] | ADR-006: ticker as text file |
| 0017 | [0.62, 0.40, 0.36, 0.10, 0.72] | ADR-007: no deletion policy |
| 0018 | [0.78, 0.70, 0.39, 0.10, 0.79] | ADR-008: file-selector as Claude tool |
| 0019 | [0.77, 0.43, 0.435, 0.10, 0.57] | Contract: file-record |
| 0020 | [0.91, 0.67, 0.27, 0.10, 0.50] | Contract: ticker-entry |
| 0021 | [0.495, 0.40, 0.22, 0.10, 0.50] | Contract: context-file |
| 0022 | [0.79, 0.54, 0.46, 0.34, 0.50] | Contract: index-entry |
| 0023 | [0.76, 0.36, 0.31, 0.10, 0.43] | Research overview |
| 0024 | [0.63, 0.27, 0.29, 0.10, 0.43] | Study 01 plan: k-value optimization |
| 0025 | [0.67, 0.20, 0.44, 0.10, 0.55] | ADR resolution log |

---

## Scores Per File

### 0002 — Synthetic test file #2
Neighbors: [0001]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0001 | 2 | Both synthetic test files; structurally similar but no content relationship |

**File mean: 2.0**

---

### 0003 — Meeting notes
Neighbors: [0002, 0001]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0002 | 2 | Both from synthetic-era of project; meeting discusses the system but not the synthetic files |
| 0001 | 1 | No relationship |

**File mean: 1.5**

---

### 0004 — File-selector implementation
Neighbors: [0001, 0002, 0003]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0001 | 2 | Synthetic test file; no impl relationship |
| 0002 | 3 | File 0002 is about 5D dimensions; proximity query uses 5D — loose but real connection |
| 0003 | 1 | Meeting notes; no relationship to implementation |

**File mean: 2.0**

---

### 0005 — Hypothesis about centrality
Neighbors: [0001, 0004, 0002, 0003]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0001 | 1 | No relationship |
| 0004 | 2 | File-selector uses centrality dimension but not the focus |
| 0002 | 4 | ADR-002 defines centrality as one of the 5 dimensions — directly relevant |
| 0003 | 1 | Meeting notes; no relationship |

**File mean: 2.0**

---

### 0006 — Architecture overview ⚠️ CRITICAL FAILURE
Neighbors: [0004, 0002, 0001, 0005, 0003]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0004 | 3 | File-selector is one component in the architecture |
| 0002 | 3 | 5D dimensions are part of the architecture |
| 0001 | 1 | Synthetic test file; no relationship |
| 0005 | 2 | Centrality hypothesis; tangential |
| 0003 | 1 | Meeting notes; no relationship |

**File mean: 2.0**

Expected top neighbors (computed distances):
- 0007 PRD: dist≈0.334 → score would be 5
- 0009 5D spec: dist≈0.369 → score would be 4
- 0010 file format: dist≈0.390 → score would be 4
- 0008 RAG vs CDS: dist≈0.402 → score would be 4

**Root cause: 0006 was indexed before files 0007–0025 existed. Stale neighbors.**

---

### 0007 — PRD source
Neighbors: [0006, 0004, 0002, 0001, 0005]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0006 | 5 | Architecture overview and PRD are directly paired |
| 0004 | 3 | File-selector is in scope of the PRD |
| 0002 | 2 | Synthetic test file; weak relationship |
| 0001 | 1 | No relationship |
| 0005 | 2 | Centrality hypothesis; tangential to PRD |

**File mean: 2.6**

---

### 0008 — RAG vs CDS
Neighbors: [0007, 0006, 0004, 0002, 0001]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0007 | 4 | PRD's motivation is the CDS vs RAG distinction |
| 0006 | 4 | Architecture overview embodies the CDS approach |
| 0004 | 3 | File-selector is a CDS-specific implementation component |
| 0002 | 3 | 5D dimensions are what differentiate CDS from RAG |
| 0001 | 1 | No relationship |

**File mean: 3.0**

---

### 0009 — 5D vector spec
Neighbors: [0008, 0007, 0006, 0004, 0002]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0008 | 4 | 5D vectors are the core of what makes CDS different from RAG |
| 0007 | 4 | 5D spec implements requirements from the PRD |
| 0006 | 4 | 5D positioning is the heart of the architecture |
| 0004 | 3 | File-selector uses 5D vectors for proximity queries |
| 0002 | 5 | File 0002 is the synthetic version of ADR-002, which defines the same 5 dimensions |

**File mean: 4.0**

---

### 0010 — File format spec
Neighbors: [0009, 0008, 0007, 0006, 0004]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0009 | 4 | File format includes the vector field; co-specs |
| 0008 | 3 | File format is part of what distinguishes CDS from RAG |
| 0007 | 4 | File format spec comes from PRD requirements |
| 0006 | 4 | File format is a core architectural element |
| 0004 | 3 | File-selector parses the file format |

**File mean: 3.6**

---

### 0011 — ADR-001: file format
Neighbors: [0002, 0004, 0001, 0003, 0005]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0002 | 4 | Adjacent ADR (ADR-002); both architectural decisions |
| 0004 | 3 | File-selector must parse files in the format defined by this ADR |
| 0001 | 1 | No relationship |
| 0003 | 1 | Meeting notes; no relationship |
| 0005 | 1 | Centrality hypothesis; no relationship |

**File mean: 2.0**

Expected top neighbors: 0010 file format spec (score 5), 0012 ADR-002 (score 5), 0013 ADR-003 (score 4)

---

### 0012 — ADR-002: 5D dimensions
Neighbors: [0011, 0002, 0001, 0004, 0005]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0011 | 5 | Adjacent ADR; directly related architectural decisions |
| 0002 | 4 | Both concern 5D dimensions (0002 is the synthetic version of this ADR) |
| 0001 | 1 | No relationship |
| 0004 | 3 | File-selector uses the 5D dimensions for proximity queries |
| 0005 | 4 | Centrality hypothesis directly addresses one of the dimensions defined here |

**File mean: 3.4**

---

### 0013 — ADR-003: heuristic-first
Neighbors: [0011, 0012, 0002, 0001, 0004]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0011 | 4 | Adjacent ADR |
| 0012 | 5 | Adjacent ADR; heuristic-first follows from 5D dimensions decision |
| 0002 | 1 | Synthetic test file; no relationship |
| 0001 | 1 | No relationship |
| 0004 | 2 | File-selector is implemented with heuristics; tangential |

**File mean: 2.6**

---

### 0014 — ADR-004: k=5
Neighbors: [0012, 0011, 0013, 0002, 0001]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0012 | 4 | k=5 is a parameter of the 5D system |
| 0011 | 3 | Both are ADRs; adjacent in series |
| 0013 | 5 | Adjacent ADR; heuristic-first directly informs k=5 |
| 0002 | 1 | Synthetic test file; no relationship |
| 0001 | 1 | No relationship |

**File mean: 2.8**

---

### 0015 — ADR-005: on-read trigger
Neighbors: [0014, 0011, 0012, 0013, 0002]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0014 | 5 | Adjacent ADR; on-read trigger decision follows from k=5 |
| 0011 | 3 | Same ADR series |
| 0012 | 3 | Same ADR series |
| 0013 | 4 | On-read trigger relates to heuristic timing |
| 0002 | 1 | Synthetic test file; no relationship |

**File mean: 3.2**

---

### 0016 — ADR-006: ticker as text file
Neighbors: [0013, 0011, 0014, 0004, 0015]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0013 | 3 | Same ADR series |
| 0011 | 3 | Same ADR series |
| 0014 | 3 | Same ADR series |
| 0004 | 3 | File-selector writes to ticker.log; implementation connects to this ADR |
| 0015 | 5 | Adjacent ADR; ticker format enables the on-read trigger |

**File mean: 3.4**

---

### 0017 — ADR-007: no deletion
Neighbors: [0016, 0015, 0011, 0014, 0013]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0016 | 4 | Adjacent ADR |
| 0015 | 3 | Same ADR series |
| 0011 | 3 | Same ADR series |
| 0014 | 3 | Same ADR series |
| 0013 | 3 | Same ADR series |

**File mean: 3.2**

---

### 0018 — ADR-008: file-selector as tool
Neighbors: [0008, 0017, 0009, 0016, 0010]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0008 | 3 | File-selector as a Claude tool relates to CDS vs RAG distinction |
| 0017 | 3 | Same ADR series |
| 0009 | 3 | File-selector tool uses 5D vectors for proximity |
| 0016 | 3 | Same ADR series |
| 0010 | 3 | File-selector tool reads the file format defined here |

**File mean: 3.0**

---

### 0019 — Contract: file-record
Neighbors: [0017, 0008, 0009, 0018, 0014]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0017 | 3 | ADR-007 (no deletion) adds the `deprecated` field to file-record |
| 0008 | 2 | RAG vs CDS; tangential |
| 0009 | 4 | 5D spec defines the vector field in file-record |
| 0018 | 3 | ADR-008 defines how file-record is returned as a tool result |
| 0014 | 2 | k=5 means neighbors list has 5 entries in file-record; indirect |

**File mean: 2.8**

Expected top neighbors: 0020 ticker-entry (score 4), 0021 context-file (score 4), 0022 index-entry (score 4)

---

### 0020 — Contract: ticker-entry
Neighbors: [0009, 0008, 0010, 0007, 0019]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0009 | 2 | 5D spec; ticker logs accesses but doesn't deal with vectors |
| 0008 | 2 | RAG vs CDS; tangential |
| 0010 | 3 | Both are data specs; ticker-entry and file format are co-infrastructure |
| 0007 | 2 | PRD loosely specifies ticker requirements |
| 0019 | 4 | Both are contracts; ticker-entry references file numbers from file-record |

**File mean: 2.6**

---

### 0021 — Contract: context-file ⚠️ ISOLATED
Neighbors: [0004, 0016, 0002, 0014, 0001]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0004 | 3 | Context-builder uses file-selector to read neighbor files |
| 0016 | 2 | Context-builder reads ticker; loose connection to ticker-format ADR |
| 0002 | 1 | Synthetic test file; no relationship |
| 0014 | 2 | Context-file includes k neighbors; indirect link to ADR-004 |
| 0001 | 1 | No relationship |

**File mean: 1.8**

Expected top neighbors: 0019 file-record (score 4), 0020 ticker-entry (score 4), 0022 index-entry (score 4)
Root cause: 0021's vector [0.495, 0.40, 0.22, 0.10, 0.50] — moderate specificity, moderate technicality — places it in the same 5D zone as synthetic test files and early ADRs, far from other contracts.

---

### 0022 — Contract: index-entry
Neighbors: [0019, 0008, 0020, 0009, 0006]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0019 | 4 | Both are contracts; index entries reference files |
| 0008 | 2 | RAG vs CDS; tangential |
| 0020 | 4 | Both are contracts for system data structures |
| 0009 | 3 | Index entries include vector status; connected to 5D spec |
| 0006 | 3 | Index entries appear in index.md, part of the architecture |

**File mean: 3.2**

---

### 0023 — Research overview ⚠️ SCATTERED
Neighbors: [0019, 0021, 0017, 0022, 0020]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0019 | 2 | File-record contract; research uses it but doesn't study it |
| 0021 | 2 | Context-file contract; same |
| 0017 | 2 | ADR-007; research references ADRs but this is not targeted |
| 0022 | 2 | Index-entry contract; same |
| 0020 | 2 | Ticker-entry contract; same |

**File mean: 2.0**

Expected top neighbors: 0024 Study 01 (score 5), 0025 ADR resolution log (score 4)
Root cause: 0023 vector [0.76, 0.36, 0.31, 0.10, 0.43] sits between the high-spec cluster and contract cluster in vector space; no other research docs were nearby at indexing time.

---

### 0024 — Study 01 plan: k-value
Neighbors: [0023, 0021, 0014, 0019, 0004]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0023 | 5 | Research overview — directly related research documents |
| 0021 | 2 | Context-file contract; study uses it indirectly |
| 0014 | 5 | ADR-004 (k=5) is the exact ADR this study tests |
| 0019 | 2 | File-record contract; indirect |
| 0004 | 2 | File-selector impl; study tests using the selector but not this specific file |

**File mean: 3.2**

---

### 0025 — ADR resolution log
Neighbors: [0024, 0014, 0019, 0023, 0015]

| Neighbor | Score | Rationale |
|----------|-------|-----------|
| 0024 | 4 | Study 01 plan feeds into ADR resolution tracking |
| 0014 | 4 | ADR-004 is one of the ADRs tracked in this log |
| 0019 | 2 | File-record contract; not related to resolution tracking |
| 0023 | 4 | Research overview is closely related to resolution status |
| 0015 | 3 | ADR-005 is another ADR in this log |

**File mean: 3.4**

---

## Aggregate Statistics

### Score Distribution

| Score | Count | % |
|-------|-------|---|
| 1 (unrelated) | 19 | 17.3% |
| 2 (weak) | 25 | 22.7% |
| 3 (tangential) | 34 | 30.9% |
| 4 (close) | 23 | 20.9% |
| 5 (direct) | 9 | 8.2% |
| **Total** | **110** | **100%** |

**Overall mean: 308 / 110 = 2.80**
**Target: > 3.5 — FAILED**

Pairs scoring ≥ 3 (relevant): 66/110 = 60.0%
Pairs scoring ≤ 2 (irrelevant): 44/110 = 40.0%

### Per-Cluster Means

| Cluster | Files | Pairs | Sum | Mean |
|---------|-------|-------|-----|------|
| Synthetic test (0001–0005) | 5 | 10 | 19 | 1.90 |
| High-spec docs (0006–0010) | 5 | 25 | 76 | 3.04 |
| ADR cluster (0011–0018) | 8 | 40 | 118 | 2.95 |
| Contracts + research (0019–0025) | 7 | 35 | 95 | 2.71 |

---

## Failure Mode Inventory

### FM-1: Stale neighbors from sequential indexing (HIGH SEVERITY)

Files indexed before most of the graph existed retain synthetic test files as neighbors, even when semantically unrelated real content has since been added.

**Affected pairs:** All pairs from 0006, 0007, and partially 0008, 0009 that point into 0001-0005.

**Evidence:** 0006 (architecture overview, [1.0,0.66,0.21,0.50,0.43]) lists all synthetic test files [0004,0002,0001,0005,0003] as its top-5, scoring 2,3,1,2,1 = mean 1.8. Computed distance to 0007 (PRD) is ≈0.334; distance to 0004 (file-selector impl) is ≈0.828. 0007 was simply absent when 0006 was indexed.

**Fix:** `kg_index_batch --force` — re-index all files against the full graph.

### FM-2: Synthetic test file gravity on ADRs (MEDIUM SEVERITY)

ADR files (0011–0018) have vectors in the range specificity 0.45–0.78, technicality 0.15–0.70. The synthetic test files (0001–0005) occupy specificity 0.30–0.35, technicality 0.20–0.36. The lower ADRs (0011–0015) have specificity that overlaps with the synthetic files, pulling them into each other's neighbor lists.

**Affected pairs:** 0011→0001, 0011→0003, 0012→0001, 0013→0001, 0013→0002, 0014→0001, 0014→0002, 0015→0002, scoring 1.

**Fix:** Re-indexing partially fixes this (FM-1 was making it worse). Longer term: if synthetic test files are deprecated, they exit proximity queries.

### FM-3: Contract cluster fragmentation (MEDIUM SEVERITY)

The four contracts should form a tight cluster, but 0021 (context-file, [0.495, 0.40, 0.22, 0.10, 0.50]) sits in a different 5D zone than the other three contracts (0019–0020, 0022), whose specificity ranges 0.77–0.91. 0021 is isolated with mean relevance 1.8.

**Root cause:** The context-file contract's vector under-estimates its specificity. The document is a detailed spec for a specific data structure — it should score higher on the specificity dimension than 0.495.

**Fix:** Manual vector correction for 0021; or adjust the specificity heuristic for contract-type documents.

### FM-4: Research cluster isolation (LOW SEVERITY)

0023 (research overview) receives all 2s from its neighbors (contracts and ADRs). 0024 and 0025 do connect to each other and to relevant ADRs, so the research cluster partially works. But 0023 is misplaced.

**Root cause:** 0023's vector [0.76, 0.36, 0.31, 0.10, 0.43] places it near the contract/ADR boundary, not near other research docs.

**Fix:** Re-indexing after 0024 and 0025 were added would likely improve 0023's neighborhood.
