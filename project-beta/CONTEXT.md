# project-beta — Task Router

## What This Project Is
[One sentence. What problem it solves. What it produces.]

---

## Task Routing

| Your Task | Go To | Also Load |
|-----------|-------|-----------|
| Define architecture or approach | `_planning/CONTEXT.md` | — |
| Make an architecture decision | `_planning/CONTEXT.md` | Relevant ADRs from `_planning/adr/` |
| Work on the CLI | `programs/cli/CONTEXT.md` | — |

---

## Before Writing Any Code

These are hard stops. Not suggestions.

**Stop 1 — Assumption ADR check:**
Read every file in `_planning/adr/`.
If any ADR has `status: assumption` and affects the program you're building → STOP.
Surface to a human. Wait for status to change to `accepted`. Then re-check.

**Stop 2 — Stub contract check:**
If this project grows to multiple programs: read every contract in `shared/contracts/`
listed in the program's External Dependencies. If any is `stub` → STOP. Define shape first.

**Stop 3 — Spec review:**
Run `../../_meta/spec-review.md` on the program.
If verdict is not `OVERALL: PASS` → STOP. Fix blocking items. Re-run. Repeat until PASS.
Only then: change the program's MANIFEST.md `status` from `scaffold` to `specced` and build.

---

## Load Rules

- Single-program project — no cross-program boundary to manage
- Still check `_planning/adr/` before making architectural choices

## Project Status
See `_planning/roadmap.md` for build order, what is complete, and what is next.
