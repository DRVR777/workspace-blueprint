# knowledge-graph — Task Router

## What This Project Is
A Cognitive Document System. Documents in Data/ are active nodes in a 5D graph.
The AI navigates by tool-use, not passive retrieval. Every read is logged.
Documents self-describe their position relative to neighbors on first access.

---

## Task Routing

| Your Task | Go To | Also Load |
|-----------|-------|-----------|
| Understand the full system | _planning/architecture.md | _planning/rag-vs-cds.md |
| Work on Data/ file format | programs/data-store/CONTEXT.md | shared/contracts/file-record.md |
| Work on the file selector tool | programs/file-selector/CONTEXT.md | shared/contracts/ticker-entry.md |
| Work on vector computation | programs/indexer/CONTEXT.md | shared/contracts/file-record.md |
| Work on embedded prompt execution | programs/context-builder/CONTEXT.md | shared/contracts/context-file.md |
| Make or review an architectural decision | _planning/adr/ | Nothing else |
| Check implementation status | _planning/roadmap.md | Nothing else |
| Define or update a contract | shared/MANIFEST.md | _planning/adr/ for context |

---

## Before Writing Any Code

**Stop 1 — ADR check:**
Read all files in `_planning/adr/`. If any has `status: assumption` and its
Consequences mentions your program — STOP. Surface to human before building.

**Stop 2 — Contract check:**
Read every contract in `shared/contracts/` your program depends on.
If any has `status: stub` — define the shape before writing code.

**Stop 3 — Spec review:**
Run `{root}/_meta/spec-review.md`. Only proceed when verdict is OVERALL: PASS.

---

## Status
All programs are `specced`. Ready to build in dependency order:
data-store → file-selector → indexer → context-builder
