<!-- See {root}/_meta/ur-prompt.md before reading this file. -->
# {{PROJECT_NAME}} — Project Map

## Hard Rule
Depth 1 only. Program names and one-line purposes.
Internal structure of each program lives in that program's own CONTEXT.md.
If program internals appear here, remove them.

---

## Programs

| Program | Purpose | Status |
|---------|---------|--------|
| [replace with program names from PRD] | [one-line purpose] | scaffold |

---

## Workspace Rules

1. An agent in one program never loads another program's src/.
2. All cross-program data shapes live in shared/contracts/.
3. Check _planning/adr/ before writing code — decisions may already be made.
4. Log every inference to _meta/gaps/pending.txt during a task, not after.
5. Fix-first: when an error or broken reference is found, fix it without asking.

---

## Navigation

| You want to... | Go to |
|----------------|-------|
| Plan or decide architecture | _planning/CONTEXT.md |
| Work on a specific program | programs/[name]/CONTEXT.md |
| Find a contract | shared/contracts/ |
| See open gaps | _meta/gaps/CONTEXT.md |
| See all architectural decisions | _planning/adr/ |
| Run spec review on a program | {root}/_meta/spec-review.md |
| Look up an architectural pattern | {root}/_core/CONVENTIONS.md |

---

## What to Load

Minimum file set per task type. Load only what is listed. Do not load output/ files unless continuing in-progress work.

| Task | Load these files | Do NOT load |
|------|-----------------|-------------|
| Start a new program | MANIFEST.md, _planning/CONTEXT.md, _planning/prd-source.md | other programs' files |
| Work on an existing program | programs/[name]/CONTEXT.md, programs/[name]/MANIFEST.md, shared/contracts/ | _planning/prd-source.md, other programs |
| Review architecture decision | _planning/adr/[relevant ADR] | program source files |
| Run spec review | programs/[name]/CONTEXT.md, programs/[name]/MANIFEST.md | output/ folders |
| Close a gap | _meta/gaps/CONTEXT.md, the gap JSON file | unrelated program files |
| Find a contract | shared/MANIFEST.md, shared/contracts/[name].md | program source files |
