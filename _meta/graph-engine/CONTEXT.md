# Graph Engine — Agent Contract

## What This Is

The materializer. It crawls the workspace, reads edge declarations from file headers,
and assembles them into `output/graph.json`. Run it whenever you need an up-to-date
view of the workspace graph (after a build session, before gap detection, or on demand).

---

## Trigger

Run this when:
- `graph` keyword is typed
- After any session that created, moved, or deleted files
- Before running gap detection (orphan check requires a fresh graph)
- After closing a gap that added a new cross-file reference

---

## Edge Declaration Format

Edges live in file headers. Any file can declare its edges. Format:

```markdown
<!--
edges:
  - type: requires
    target: path/to/file.md
  - type: enables
    target: path/to/other-file.md
-->
```

Or in JSON front-matter:
```json
{
  "edges": [
    { "type": "requires", "target": "path/to/file.md" },
    { "type": "enables", "target": "path/to/other-file.md" }
  ]
}
```

**Edge types:**

| Type | Meaning |
|------|---------|
| `requires` | This file cannot function without the target |
| `instantiates` | This file creates an instance of the target's schema/contract |
| `enables` | This file's existence makes the target possible but not required |
| `contradicts` | This file conflicts with the target — one of them may be wrong |
| `refines` | This file is a more specific version of the target |

**Direction:** Edges are directional. `A requires B` means A depends on B — not the reverse.

---

## Process

### Step 1 — Crawl
Scan all `.md` and `.json` files in the workspace root recursively.
Skip: `output/` folders, `leftOffHere/`, `_examples/`.

For each file:
1. Read the first 30 lines (headers only — do not read full file)
2. Extract any `edges:` declarations
3. Record: `{ source: file_path, type: edge_type, target: target_path }`

### Step 2 — Resolve Targets
For each edge, verify the target file exists.
- Target exists → mark edge `status: valid`
- Target does not exist → mark edge `status: broken`, log to `_meta/gaps/pending.txt`:
  `[timestamp] | [source_file] | graph-engine | inferred "broken edge: [source] --[type]--> [target] (target not found)"`

### Step 3 — Build Node List
Every file referenced as a source or target is a node.
Node fields: `{ id: file_path, type: inferred_from_extension, depth: folder_depth, edge_count: N }`

### Step 4 — Detect Orphans
An orphan is a node with `edge_count: 0` (no incoming or outgoing edges).
For each orphan, log to `_meta/gaps/pending.txt`:
  `[timestamp] | [file_path] | graph-engine | inferred "orphan node: no edges declared — may be disconnected"`

Exception: known leaf files do not need edges (content files in `output/`, ADR files, queue files).
Leaf file patterns: `output/**`, `_intake/queue/**`, `_intake/processed/**`, `leftOffHere/**`.

### Step 5 — Write graph.json
Write `_meta/graph-engine/output/graph.json` using graph-schema.json format.
Overwrite any existing file — this is always a full rebuild.

---

## Outputs

| Output | Location |
|--------|----------|
| Materialized graph | `_meta/graph-engine/output/graph.json` |
| Broken edge entries | `_meta/gaps/pending.txt` |
| Orphan node entries | `_meta/gaps/pending.txt` |

---

## Audit

Before marking the run complete:
- [ ] All `.md` files in `programs/` were scanned (spot-check 3)
- [ ] Broken edges were logged to pending.txt (count matches graph.json broken_edge_count)
- [ ] Orphan count in graph.json matches orphan entries in pending.txt
- [ ] `graph.json` validates against graph-schema.json
- [ ] `output/` folder exists and graph.json was written
