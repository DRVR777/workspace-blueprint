# Implementation Roadmap — knowledge-graph

Status: ✅ done | ⚠️ partial | ❌ not built | 🔒 blocked

---

## Build Order

Programs must be built in dependency order. A program cannot be built until
all programs it depends on are ✅.

```
data-store (no deps)
    ↓
file-selector (depends on: data-store)
    ↓
indexer (depends on: data-store)
    ↓
context-builder (depends on: data-store + file-selector + indexer)
```

---

## Program 1 — data-store

| Feature | Status | Notes |
|---------|--------|-------|
| File creation with correct naming (file0001.md...) | ✅ | data_store.py create |
| Metadata header generation | ✅ | All 7 required fields |
| Standard embedded prompt template injection | ✅ | Per-file number substitution |
| Counter management (no gaps, no duplicates) | ✅ | Scans existing files, takes max+1 |
| index.md maintenance | ✅ | Append on create, update on deprecate |
| File format validator | ✅ | data_store.py validate — rules 1–6 from spec |
| Deprecation support | ✅ | data_store.py deprecate — ADR-007 |
| Bulk import (convert existing docs to CDS format) | ❌ | Low priority |

---

## Program 2 — file-selector

| Feature | Status | Notes |
|---------|--------|-------|
| Read file by number | ✅ | file_selector.py read |
| Append to ticker.log on read | ✅ | Every read, no exceptions |
| Increment access_count in file metadata | ✅ | In-place update |
| Proximity query (5D coordinates → k nearest) | ✅ | Euclidean + weighted distance; skips null-vector files |
| Session ID tracking | ✅ | Passed in CLI or tool_use input |
| Return file content + metadata separately | ✅ | Matches file-record.md contract |
| Error: file not found | ✅ | Structured error dict, no unhandled exception |
| Claude tool definition (tool_use JSON schema) | ✅ | TOOL_SCHEMA constant; `schema` CLI command |
| needs_indexing flag | ✅ | Set on return when vector contains nulls |

---

## Program 3 — indexer

| Feature | Status | Notes |
|---------|--------|-------|
| Heuristic vector computation (5 dimensions) | ✅ | All 5 scorers implemented |
| k-nearest neighbor computation | ✅ | Euclidean distance in 5D |
| Write vector + neighbors back to file metadata | ✅ | In-place update |
| Update last_indexed timestamp | ✅ | |
| Batch mode (index all null-vector files) | ✅ | indexer.py batch |
| Force re-index | ✅ | indexer.py reindex / batch --force |
| Centrality update from ticker.log | ✅ | Co-access session analysis |
| Watch mode (index new files automatically) | ❌ | Low priority |
| ML projection upgrade path | ❌ | Future — not in initial build |

---

## Program 4 — context-builder

| Feature | Status | Notes |
|---------|--------|-------|
| Watch ticker.log for new entries | ✅ | context_builder.py watch (polls, Ctrl-C to stop) |
| Read file's neighbors via file-selector | ✅ | Reads each neighbor file directly |
| Write ctx-NNNN.md (What I Am, Position, Neighbors, Cluster, Role) | ✅ | All 5 sections |
| Update existing ctx file when file is re-read | ✅ | Overwrites, not appends |
| Append `context_built` to ticker.log after writing | ✅ | Every write |
| Skip if ctx file is recent (< N minutes old) | ✅ | Configurable --staleness (default 5 min) |
| Force rebuild | ✅ | --force flag |
| Status overview | ✅ | context_builder.py status |
| Note: heuristic execution only | — | LLM-quality ctx requires Claude API session |

---

## Shared Infrastructure

| Feature | Status | Notes |
|---------|--------|-------|
| shared/contracts/file-record.md | ✅ | Shape of a file record |
| shared/contracts/ticker-entry.md | ✅ | Shape of a ticker log entry |
| shared/contracts/context-file.md | ✅ | Shape of ctx-NNNN.md |
| shared/contracts/index-entry.md | ✅ | Shape of a row in index.md |
| Data/ folder bootstrap script | ✅ | data_store.py init |

---

## ADRs Required Before Building

| ADR | Decision | Status |
|-----|----------|--------|
| ADR-001 | File format (YAML header vs custom) | ✅ accepted |
| ADR-002 | 5D dimensions definition | ✅ accepted |
| ADR-003 | Indexer computation method (heuristic first) | ✅ accepted |
| ADR-004 | k value for nearest neighbors | ✅ accepted — k=5 |
| ADR-005 | Context-builder trigger (on-read vs batch) | ✅ accepted — on-read |
| ADR-006 | Ticker format (file vs DB) | ✅ accepted — file |
| ADR-007 | File deletion policy | ✅ accepted — deprecate only |
| ADR-008 | file-selector as Claude tool | ✅ accepted — tool_use, MCP future |

All ADRs accepted. Build is unblocked.

---

## Build Session 1 — Data Layer Foundation

Goal: Get Data/ folder working with correct file format.

Tasks:
1. Write shared/contracts/ (all 4 contracts)
2. Build data-store: file creation, naming, metadata, embedded prompt template
3. Bootstrap Data/ folder (index.md, ticker.log, first test file)
4. Validate: manually inspect file0001.md matches file-format-spec.md exactly

Deliverable: `Data/file0001.md` exists, format is correct, index.md has one row.

---

## MCP Server

| Feature | Status | Notes |
|---------|--------|-------|
| kg_read | ✅ | Read by file number, logs ticker, increments access_count |
| kg_query | ✅ | 5D proximity search, returns k nearest |
| kg_create | ✅ | Next sequential file, metadata + embedded prompt |
| kg_index | ✅ | Heuristic 5D vector + k-NN |
| kg_index_batch | ✅ | Index all pending files |
| kg_build_ctx | ✅ | Generate ctx-NNNN.md, returns content |
| kg_validate | ✅ | Check all files against spec |
| kg_status | ✅ | File count, indexed count, ticker size |
| Registered globally | ✅ | User-scope, all Claude Code sessions |

---

## Build Session 2 — Navigation

Goal: AI can read files and navigation is logged.

Tasks:
1. Build file-selector: read by number, write to ticker, return content
2. Define file-selector as a Claude tool (ADR-008)
3. Test: call file-selector("0001"), verify ticker.log has entry
4. Test: call file-selector with 5D query (placeholder — indexer not built yet)

Deliverable: AI can read Data/file0001.md via tool. Ticker has the read event.

---

## Build Session 3 — Positioning

Goal: Every file has a meaningful 5D position.

Tasks:
1. Build indexer: heuristic computation for all 5 dimensions
2. Build k-nearest neighbor search
3. Update file metadata with vector + neighbors
4. Test: file0001.md shows vector: [0.x, 0.x, 0.x, 0.x, 0.x] and neighbors: [...]

Deliverable: file0001.md has a non-null vector. Proximity query in file-selector works.

---

## Build Session 4 — Self-Description

Goal: Files describe themselves when read.

Tasks:
1. Build context-builder: watch ticker, run embedded prompt, write ctx files
2. Test: read file0001.md via file-selector → ctx-0001.md is written
3. Test: read file0001.md again → ctx-0001.md is updated (not duplicated)
4. Verify ctx-0001.md has all required sections

Deliverable: Reading any file produces a context file describing its position in the graph.

---

## Done = Full Loop Working

```
1. Create file → data-store assigns number, writes format
2. Index file → indexer assigns 5D vector, finds neighbors
3. Read file → file-selector logs to ticker
4. Embedded prompt fires → context-builder writes ctx file
5. Read ctx file → AI has description of where this doc sits in the graph
6. Read neighbors → repeat from step 3
```

When this loop runs end-to-end on 10 files, the system is working.
