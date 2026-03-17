# Architecture — Cognitive Document System

## System Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  AI AGENT (Claude)                                          │
│                                                             │
│  "I want to understand topic X"                             │
│      ↓                                                      │
│  [calls file-selector tool with file number or 5D query]   │
│      ↓                                    ↑                 │
│  Reads file content + embedded prompt     │                 │
│      ↓                                    │                 │
│  Embedded prompt: "read my neighbors"     │                 │
│      ↓                                    │                 │
│  [calls file-selector for each neighbor] ─┘                 │
│      ↓                                                      │
│  context-builder writes ctx-NNNN.md                        │
└─────────────────────────────────────────────────────────────┘
                    │               ↑
                    ↓               │
┌─────────────────────────────────────────────────────────────┐
│  DATA LAYER                                                 │
│                                                             │
│  Data/                                                      │
│    file0001.md  ← [metadata][5D vector][neighbors]         │
│    file0002.md     [embedded prompt]                        │
│    file0003.md     [content]                                │
│    ...                                                      │
│    ctx-0001.md  ← context file (written by context-builder)│
│    ctx-0002.md                                              │
│    ticker.log   ← append-only access log                   │
│    index.md     ← directory: file number → one-line summary│
└─────────────────────────────────────────────────────────────┘
                    ↑
         ┌──────────┴──────────┐
         │                     │
┌────────────────┐  ┌─────────────────────┐
│  INDEXER       │  │  CONTEXT-BUILDER     │
│                │  │                     │
│ Reads content  │  │ Watches ticker.log  │
│ Assigns 5D     │  │ Runs embedded prompt│
│ vector         │  │ Reads neighbors     │
│ Finds k nearest│  │ Writes ctx-NNNN.md  │
│ Updates file   │  │                     │
│ metadata       │  └─────────────────────┘
└────────────────┘
```

---

## Data Flow

### Write path (new file)
```
1. data-store assigns next file number (NNNN)
2. data-store creates file0NNNN.md with:
   - metadata header (vector=null, neighbors=[])
   - embedded prompt (standard template)
   - content (whatever the file is about)
3. data-store updates Data/index.md with the new entry
4. indexer detects new file (or is called explicitly)
5. indexer computes 5D vector from content
6. indexer finds k nearest neighbors in existing vector space
7. indexer writes vector + neighbors back to file metadata
```

### Read path (AI navigates)
```
1. AI calls file-selector("0042")
2. file-selector reads Data/file0042.md
3. file-selector appends to Data/ticker.log:
   [timestamp] | 0042 | [session] | [reason]
4. file-selector returns file content to AI
5. AI reads embedded prompt
6. Embedded prompt instructs: read neighbors [0031, 0051, 0019]
7. AI calls file-selector for each neighbor (steps 1-5 repeat)
8. context-builder detects new ticker entries
9. context-builder writes Data/ctx-0042.md:
   - What file0042 is about
   - How it relates to 0031, 0051, 0019
   - Its cluster/topic label
   - Its role sentence
```

### Query path (find by 5D position)
```
1. AI calls file-selector with 5D coordinates instead of file number
2. file-selector scans all file vectors
3. Returns k nearest files (lowest Euclidean distance in 5D space)
4. Normal read path continues for each returned file
```

---

## Component Responsibilities (strict boundaries)

| Component | Owns | Never touches |
|-----------|------|---------------|
| data-store | Data/ folder structure, file creation, naming, index.md | Vector computation, ticker |
| file-selector | File access, ticker.log | File creation, vector computation |
| indexer | Vector field in file metadata, neighbors field | Context files, ticker |
| context-builder | ctx-NNNN.md files | File creation, vector computation, ticker writes |

**Rule:** No component reads another component's internal code.
All cross-component data flows through the shared contracts.

---

## Contracts (see shared/contracts/)

| Contract | Produced by | Consumed by |
|----------|-------------|-------------|
| file-record.md | data-store | file-selector, indexer, context-builder |
| ticker-entry.md | file-selector | context-builder |
| context-file.md | context-builder | (AI reads directly) |
| index-entry.md | data-store | (AI reads directly) |

---

## The Ticker as Emergent Graph

The ticker starts as a simple log. Over time it becomes a graph:

- **Node weight** = access count per file (frequently read = important)
- **Edge** = two files read in the same session within N reads of each other
- **Path** = sequence of reads in one session = one traversal of the graph
- **Cluster** = files frequently co-accessed = implicitly related

The `graph_builder.py` script (in `{root}/_meta/scripts/` — to be extended) can
materialize `ticker.log` into `graph.json` for visualization or analysis.

---

## Session Lifecycle

```
Session starts
    ↓
AI has a goal (explore topic, answer question, build something)
    ↓
AI calls file-selector with starting file or 5D query
    ↓
AI reads files, follows embedded prompts, reads neighbors
    ↓
context-builder runs after each read (async or triggered)
    ↓
AI builds understanding from the files + context files it has read
    ↓
Session ends — ticker.log has a complete record of the traversal
    ↓
(next session starts from ticker — AI can resume where it left off)
```

---

## Scalability Properties

- **No upper bound on file count** — sequential numbering scales to file99999+
- **No restructuring on growth** — adding files never changes existing files
- **Self-healing** — if a context file is stale, embedded prompt rewrites it on next read
- **Self-indexing** — indexer can run as a background pass over all null-vector files
- **Portable** — entire system is a folder of text files. No database. No server.

---

## What the 5D Space Looks Like

Five dimensions, each 0.0–1.0. Visualizable as a 5D hypercube.
In practice: cluster by first two dimensions for a 2D visualization.

Example cluster map:
```
High specificity (dim1 → 1.0) + High technicality (dim2 → 1.0):
    → Code files, API specs, schemas

Low specificity (dim1 → 0.0) + Low technicality (dim2 → 0.0):
    → Principles, philosophies, visions

Low temporality (dim3 → 0.0) + High confidence (dim5 → 1.0):
    → Architectural decisions, accepted ADRs

High temporality (dim3 → 1.0) + Low confidence (dim5 → 0.0):
    → Meeting notes, open questions, brainstorms
```
