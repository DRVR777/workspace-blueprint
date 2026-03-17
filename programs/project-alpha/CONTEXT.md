# project-alpha — Task Router

## What This Project Is
[One sentence. What problem it solves. What it produces.]

---

## Before Writing Any Code

These are hard stops. Not suggestions.

**Stop 1 — Assumption ADR check:**
Read every file in `_planning/adr/`.
If any ADR has `status: assumption` and affects the program you're building → STOP.
Surface to a human. Wait for status to change to `accepted`. Then re-check.

**Stop 2 — Stub contract check:**
Read every contract in `shared/contracts/` listed in the program's External Dependencies.
If any contract has `status: stub` → STOP.
Define the shape before writing code that produces or consumes it.

**Stop 3 — Spec review:**
Run `{root}/_meta/spec-review.md` on the target program.
If verdict is not `OVERALL: PASS` → STOP. Fix blocking items. Re-run. Repeat until PASS.
Only then: change the program's MANIFEST.md `status` from `scaffold` to `specced` and build.

---

## Task Routing

| Your Task | Go To | Also Load |
|-----------|-------|-----------|
| Define system architecture | `_planning/CONTEXT.md` | — |
| Make an architecture decision | `_planning/CONTEXT.md` | Relevant ADRs from `_planning/adr/` |
| Define a contract between programs | `shared/MANIFEST.md` | Existing contracts in `shared/contracts/` |
| Work on the API | `programs/api/CONTEXT.md` | Relevant contract from `shared/contracts/` |
| Work on the frontend | `programs/frontend/CONTEXT.md` | Relevant contract from `shared/contracts/` |
| Debug a cross-program issue | Start at `shared/contracts/` | Both programs' CONTEXT.md files |

---

## Load Rules

- Never load two programs' docs simultaneously unless debugging a cross-program boundary
- Always check `_planning/adr/` before making architectural choices — the decision may already be made
- Contracts in `shared/` are the source of truth — if code disagrees with the contract, the code is wrong

---

## Project Status
See `_planning/roadmap.md` for build order, what is complete, and what is next.
