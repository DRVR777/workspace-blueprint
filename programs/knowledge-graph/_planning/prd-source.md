# PRD Source — knowledge-graph (Cognitive Document System)

Origin: Conversation with user, 2026-03-13.
The user described this system incrementally. This document reconstructs the full spec
from that description plus architectural inference. All inferences are logged as ADRs.

---

## Core Premise

A folder of files (`Data/`) where each file is:
1. **Sequentially named** — `file0001.md`, `file0002.md`, etc.
2. **Position-aware** — carries a 5D vector describing its location in semantic space
3. **Active** — carries an embedded prompt that fires when the AI reads the file
4. **Self-orienting** — the embedded prompt looks at neighbors and writes a context file

The AI navigates this system by **tool-use** — it calls a `file-selector` tool to read
documents. It does not receive documents passively. It chooses what to read.

Every file read is appended to a **global ticker** — an append-only log of file accesses.
This creates an attention trace: which documents the AI has visited, in what order.

---

## The Four Programs

### 1. data-store
Owns the `Data/` folder. Responsible for:
- Creating new files with correct naming convention (`file[NNNN].md`)
- Enforcing the file format (metadata header + embedded prompt + content)
- Maintaining a directory index (`Data/index.md`) listing all files with one-line summaries
- Incrementing the file counter correctly (no gaps, no duplicates)

### 2. file-selector
The AI's tool for reading documents. Responsible for:
- Accepting a file number (or list of numbers) as input
- Reading and returning the file content
- **Appending to the global ticker** (`Data/ticker.log`) on every read
- Optionally: accepting a 5D position and returning the k nearest neighbors

This is a **Claude tool** (function/tool_use) — not a script run by a human.
The AI calls this tool directly during a session.

### 3. indexer
Computes 5D vector positions for each file. Responsible for:
- Reading file content
- Assigning a 5D coordinate based on the 5 defined dimensions
- Writing the vector back to the file's metadata header
- Maintaining a neighbor list for each file (k-nearest in 5D space)
- Running on new files and re-running when file content changes significantly

### 4. context-builder
Executes embedded prompts. Responsible for:
- Detecting when a file has been read (by watching the ticker)
- Running the embedded prompt for that file
- Reading the file's neighbor list (from indexer output)
- Writing/updating `Data/ctx-[NNNN].md` — the context file for that document
- The context file describes: what this document is, how it relates to neighbors,
  what cluster it belongs to, its role in the graph

---

## File Format Spec (see `file-format-spec.md` for full detail)

Every file in `Data/` follows this structure:

```
---
filename: NNNN
vector: [null, null, null, null, null]
neighbors: []
context_file: Data/ctx-NNNN.md
created: YYYY-MM-DD
last_indexed: null
---

<!-- EMBEDDED PROMPT — EXECUTE ON READ -->
You are reading file NNNN in a self-navigating knowledge graph.

Your task when you read this file:
1. Read my neighbors list above. For each neighbor, call file-selector with that number.
2. After reading neighbors, write or update Data/ctx-NNNN.md with:
   a. What I (file NNNN) am about — one paragraph
   b. How I relate to each neighbor — one sentence each
   c. What cluster/topic area I belong to — one label
   d. My role: "This document is a [noun] that [verb phrase]."
3. If my vector field is [null, null, null, null, null], request indexing.
<!-- END EMBEDDED PROMPT -->

[document content starts here]
```

---

## The 5D Vector (see `5d-vector-spec.md` for full detail)

Each dimension is a float from 0.0 to 1.0:

| Dim | Name | 0.0 means | 1.0 means |
|-----|------|-----------|-----------|
| 1 | **Specificity** | Abstract/general | Concrete/specific |
| 2 | **Technicality** | Conceptual/intuitive | Technical/formal |
| 3 | **Temporality** | Foundational/stable | Current/ephemeral |
| 4 | **Centrality** | Peripheral node | Central hub |
| 5 | **Confidence** | Speculative/uncertain | Established/verified |

Example: a foundational architectural decision would be `[0.8, 0.5, 0.1, 0.9, 0.9]`
— specific, moderately technical, stable, central, highly confident.

A brainstorm note would be `[0.3, 0.2, 0.8, 0.1, 0.2]`
— abstract, intuitive, current, peripheral, speculative.

---

## The Global Ticker

File: `Data/ticker.log` — append-only, never edited.

Format:
```
[ISO-8601] | [file_number] | [session_id] | [reason]
```

Example:
```
2026-03-13T14:22:01Z | 0001 | session-abc123 | initial_read
2026-03-13T14:22:15Z | 0003 | session-abc123 | neighbor_of_0001
2026-03-13T14:22:31Z | 0007 | session-abc123 | neighbor_of_0001
2026-03-13T14:23:05Z | 0003 | session-abc123 | revisit
```

The ticker enables:
- **Attention analysis** — which files are hot (frequently accessed)
- **Path replay** — reconstruct what the AI was exploring
- **Loop detection** — AI keeps returning to same files = needs new information
- **Navigation graph** — edges formed by (file_A → file_B in same session) pairs

---

## What This Is Not

See `rag-vs-cds.md` for the full comparison. Short version:

| Property | RAG | This System (CDS) |
|----------|-----|-------------------|
| Retrieval trigger | External query | AI tool-use (active choice) |
| Document role | Passive storage | Active participant (embedded prompt) |
| Navigation | Query → results | AI chooses path via selector |
| Memory | None | Global ticker = attention trace |
| Self-description | None | Context files written by embedded prompts |
| Vector dimensions | 768-4096 | 5 (interpretable) |

---

## Open Questions (logged as ADRs)

1. How does the indexer compute 5D positions? (heuristic vs ML) → ADR-003
2. What is k for k-nearest neighbors? → ADR-004
3. When does context-builder run? (on read vs on index) → ADR-005
4. How is ticker.log shared across sessions? (file vs DB) → ADR-006
5. Can files be deleted or only deprecated? → ADR-007
