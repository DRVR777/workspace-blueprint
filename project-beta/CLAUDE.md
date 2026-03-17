# project-beta — Project Map

## Hard Rule
This file maps project-beta's depth 1 only. Program names and one-line purposes.
Internal structure of each program lives in that program's own CONTEXT.md.

---

## Programs

| Program | Purpose | Status |
|---------|---------|--------|
| `programs/cli/` | [e.g. Command-line tool — single program, no inter-service dependencies] | scaffold |

---

## Planning

Architecture decisions and roadmap live in `_planning/`.
Build nothing without checking `_planning/adr/` for relevant decisions first.

---

## Shared

If this project grows to multiple programs, contracts go in `shared/`.
Single-program projects may not need `shared/` — delete it if unused.

---

## Navigation

| You want to... | Go to |
|----------------|-------|
| Make an architecture decision | `_planning/CONTEXT.md` |
| Build the CLI | `programs/cli/CONTEXT.md` |
| Log a project-internal gap | `_meta/gaps/pending.txt` |
| Log a cross-project gap | root `_meta/gaps/pending.txt` |
