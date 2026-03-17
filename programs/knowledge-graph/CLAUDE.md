<!-- See {root}/_meta/ur-prompt.md before reading this file. -->
# knowledge-graph — Project Map

## Hard Rule
Depth 1 only. Program names and one-line purposes.

---

## Programs

| Program | Purpose | Status |
|---------|---------|--------|
| `programs/data-store/` | Manages the Data/ folder — file creation, naming, format enforcement | specced |
| `programs/file-selector/` | The AI's tool for reading documents — appends to global ticker on every read | specced |
| `programs/indexer/` | Computes and writes 5D vector positions for each file | specced |
| `programs/context-builder/` | Executes embedded prompts — reads neighbors, writes ctx-NNNN.md files | specced |

---

## Build Order

```
1. data-store    ← file format must exist before anything else can read/write files
2. file-selector ← tool must exist before AI can navigate
3. indexer       ← vectors needed before context-builder can find neighbors
4. context-builder ← runs last; depends on files + vectors + selector being in place
```

---

## What to Load

| Task | Load these files | Do NOT load |
|------|-----------------|-------------|
| Understand the system | _planning/architecture.md | program src/ files |
| Work on data-store | programs/data-store/CONTEXT.md, shared/contracts/file-record.md | other programs |
| Work on file-selector | programs/file-selector/CONTEXT.md, shared/contracts/ticker-entry.md | data-store src/ |
| Work on indexer | programs/indexer/CONTEXT.md, shared/contracts/file-record.md | context-builder src/ |
| Work on context-builder | programs/context-builder/CONTEXT.md, shared/contracts/context-file.md | indexer src/ |
| Check implementation status | _planning/roadmap.md | — |
| Resolve an architectural decision | _planning/adr/[relevant] | program src/ |

---

## Workspace Rules

1. An agent in one program never loads another program's src/.
2. All cross-program data shapes live in shared/contracts/.
3. Check _planning/adr/ before writing code — most decisions are already made.
4. Log every inference to _meta/gaps/pending.txt during the task, not after.
5. Fix-first: when an error or broken reference is found, fix it without asking.

---

## Navigation

| You want to... | Go to |
|----------------|-------|
| See why this is not RAG | _planning/rag-vs-cds.md |
| Understand full architecture | _planning/architecture.md |
| See the file format spec | _planning/file-format-spec.md |
| Understand the 5D vector | _planning/5d-vector-spec.md |
| See all architectural decisions | _planning/adr/ |
| Check what's built | _planning/roadmap.md |
