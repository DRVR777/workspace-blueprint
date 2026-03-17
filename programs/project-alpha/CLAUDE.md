<!-- See {root}/_meta/ur-prompt.md before reading this file. -->
# project-alpha — Project Map

## Hard Rule
This file maps project-alpha's depth 1 only. Program names and one-line purposes.
Internal structure of each program lives in that program's own CONTEXT.md.

---

## Programs

| Program | Purpose | Status |
|---------|---------|--------|
| `programs/api/` | REST API — authentication, data, business logic | scaffold |
| `programs/frontend/` | Web client — user interface and interactions | scaffold |

---

## Planning

Architecture decisions, system design, and roadmap live in `_planning/`.
Build nothing without checking `_planning/adr/` for relevant decisions first.

---

## Shared

Contracts and types shared between programs live in `shared/`.
If two programs need to agree on a data shape — it goes in `shared/contracts/`, not in either program.

---

## Cross-Program Dependencies

| Consumer | Depends On | Contract |
|----------|-----------|----------|
| programs/frontend/ | programs/api/ | shared/contracts/ (when defined) |

---

## Navigation

| You want to... | Go to |
|----------------|-------|
| Make an architecture decision | `_planning/CONTEXT.md` |
| Build in a specific program | `programs/[name]/CONTEXT.md` |
| Check or define a contract | `shared/MANIFEST.md` |
| Log a project-internal gap | `_meta/gaps/pending.txt` |
| Log a cross-project gap | `{root}/_meta/gaps/pending.txt` |
