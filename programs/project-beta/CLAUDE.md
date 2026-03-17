<!-- See {root}/_meta/ur-prompt.md before reading this file. -->
# project-beta — Project Map

## Hard Rule
Depth 1 only. Program names and one-line purposes.

---

## Programs

| Program | Purpose | Status |
|---------|---------|--------|
| `programs/cli/` | Command-line tool — single program, no inter-service dependencies | scaffold |

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
| Log a cross-project gap | `{root}/_meta/gaps/pending.txt` |
