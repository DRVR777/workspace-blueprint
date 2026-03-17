# File Format Spec — Data/ documents

Every file in `Data/` follows this exact structure. No exceptions.
data-store enforces this format on creation. validate_manifests.py equivalent
for Data/ files is `data-store`'s validate command.

---

## Naming Convention

```
Data/file[NNNN].md
```

- `NNNN` = zero-padded 4-digit integer: `0001`, `0002`, ..., `9999`
- Extension: `.md` always
- No other characters in filename
- Counter is global — never reused, never skipped

For scale beyond 9999: extend to 5 digits (`00001`) — data-store handles the transition.

Context files follow the same counter:
```
Data/ctx-[NNNN].md
```

---

## File Structure

```markdown
---
filename: NNNN
vector: [null, null, null, null, null]
neighbors: []
context_file: Data/ctx-NNNN.md
created: YYYY-MM-DD
last_indexed: null
access_count: 0
---

<!-- EMBEDDED PROMPT — EXECUTE ON READ -->
[embedded prompt — see below]
<!-- END EMBEDDED PROMPT -->

[content]
```

### Metadata Header

Between the `---` fences. YAML-like but not required to be parsed by YAML.
The metadata parser reads line by line: `key: value`.

| Field | Type | Initial value | Set by |
|-------|------|--------------|--------|
| `filename` | int (4 chars) | NNNN | data-store |
| `vector` | list of 5 floats or nulls | [null, null, null, null, null] | indexer |
| `neighbors` | list of file numbers | [] | indexer |
| `context_file` | string path | Data/ctx-NNNN.md | data-store |
| `created` | ISO date | today | data-store |
| `last_indexed` | ISO datetime or null | null | indexer |
| `access_count` | int | 0 | file-selector (increments on each read) |

### Vector Format

```yaml
vector: [0.8, 0.3, 0.2, 0.9, 0.7]
```

Five floats, all between 0.0 and 1.0 inclusive.
`null` means not yet indexed. file-selector requests indexing when it reads a null vector.

### Neighbors Format

```yaml
neighbors: [0031, 0019, 0051, 0004, 0072]
```

Ordered by ascending distance in 5D space (closest first).
Default k=5. Configurable via indexer.
Empty list `[]` means not yet indexed.

---

## Embedded Prompt

The embedded prompt is the same for all files by default.
data-store writes the standard template. Domain-specific overrides are allowed
(write a custom prompt in the content area and delete the standard one).

### Standard Embedded Prompt

```markdown
<!-- EMBEDDED PROMPT — EXECUTE ON READ -->
You are reading file [NNNN] in a self-navigating knowledge graph.

When you read this file, do the following:

1. CHECK VECTOR: If my vector field shows nulls, call indexer to compute my position.
   Do this before proceeding.

2. READ NEIGHBORS: My neighbors are listed above. For each neighbor number:
   Call file-selector with that number. Read the returned content.
   If I have no neighbors yet, skip to step 3.

3. WRITE/UPDATE CONTEXT FILE: Write or overwrite Data/ctx-[NNNN].md with:

   ## What I Am
   [1 paragraph: what this document is about]

   ## My Position
   Vector: [values] — interpreted as:
   - Specificity: [low/medium/high]
   - Technicality: [low/medium/high]
   - Temporality: [stable/mixed/current]
   - Centrality: [peripheral/connected/hub]
   - Confidence: [speculative/probable/established]

   ## My Neighbors and How I Relate to Them
   [For each neighbor: file number, one-sentence relationship]

   ## My Cluster
   [One label for the topic area I belong to]

   ## My Role
   "This document is a [noun] that [verb phrase]."

4. LOG: After writing ctx-[NNNN].md, append to Data/ticker.log:
   [timestamp] | [NNNN] | [session] | context_built
<!-- END EMBEDDED PROMPT -->
```

### Custom Embedded Prompts

Some files may have specialized prompts. Examples:
- **Index node**: "You are a hub. Read all your neighbors. Synthesize them into a summary."
- **Entry point**: "You are the recommended starting file. Help the reader decide where to go next."
- **Deprecated**: "This file is deprecated. Read [NNNN] instead."

Custom prompts must still write to ctx-[NNNN].md.

---

## Context File Format (Data/ctx-NNNN.md)

Written by context-builder, readable by any agent directly.

```markdown
# Context — file[NNNN]

Generated: [ISO-8601 timestamp]
Vector: [values]

## What I Am
[paragraph]

## My Position
[interpretation of 5D coordinates]

## My Neighbors and How I Relate to Them
| Neighbor | Relationship |
|----------|-------------|
| file[N] | [one sentence] |
| file[N] | [one sentence] |

## My Cluster
[label]

## My Role
"This document is a [noun] that [verb phrase]."
```

---

## Index File (Data/index.md)

Maintained by data-store. One row per file.

```markdown
# Data/ Index

| File | Created | Vector Status | One-Line Summary |
|------|---------|--------------|-----------------|
| file0001 | 2026-03-13 | indexed | [first sentence of content] |
| file0002 | 2026-03-13 | pending | [first sentence of content] |
```

---

## Ticker Log (Data/ticker.log)

Append-only. Never edited. Written by file-selector.

```
[ISO-8601] | [file_number] | [session_id] | [reason]
```

Valid reasons:
- `direct_read` — AI called file-selector with this number explicitly
- `neighbor_of_[N]` — AI read this because it's a neighbor of file N
- `proximity_query` — AI found this via 5D proximity search
- `revisit` — AI has read this file before in this session
- `context_built` — context-builder finished ctx-[NNNN].md
- `indexed` — indexer assigned vector and neighbors

---

## Validation Rules

A file is valid if:
1. Filename matches `file[NNNN].md` pattern
2. Metadata header is present and complete (all fields exist)
3. Embedded prompt is present (between the comment markers)
4. `access_count` is a non-negative integer
5. `vector` is either all nulls or all floats in [0.0, 1.0]
6. `neighbors` is a list of valid file numbers (files that exist in Data/)
