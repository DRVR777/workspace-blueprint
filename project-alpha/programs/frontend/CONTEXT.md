# programs/frontend — Task Router

## What This Program Is
[One sentence: what it renders and who uses it.]

---

## Before Writing Any Code

**Stop 1 — Assumption ADR check:**
Read every file in `../../_planning/adr/`.
If any ADR has `status: assumption` and affects this program → STOP.
Surface to a human. Wait for `accepted`. Then re-check.

**Stop 2 — Stub contract check:**
Read every contract in this program's MANIFEST.md `External Dependencies` table.
If any contract at `../../shared/contracts/[contract]` has `status: stub` → STOP.
Define the shape before writing code that consumes it.

**Stop 3 — Spec review:**
Run `../../_meta/spec-review.md` on this program.
If verdict is not `OVERALL: PASS` → STOP. Fix blocking items. Re-run. Repeat until PASS.

---

## Task Routing

| Your Task | Load These | Skip These |
|-----------|-----------|------------|
| Build a new UI feature | MANIFEST.md, relevant contract from `../../shared/contracts/` | Other programs' src/ |
| Fix a visual or logic bug | The failing component file, its contract | _planning/ (unless fix changes an interface) |
| Write tests | Component under test, its contract | Everything else |
| Update an API-consumed interface | `../../shared/contracts/[contract]`, `../../_planning/adr/` | src/ until contract updated |

---

## Inference Logging
All inferences → `../../_meta/gaps/pending.txt`
Format: `[timestamp] | frontend | [file] | inferred "[what]" — no file states this`
