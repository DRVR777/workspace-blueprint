# programs/cli — Task Router

## What This Program Is
[One sentence: what commands it exposes and who runs it.]

---

## Before Writing Any Code

**Stop 1 — Assumption ADR check:**
Read every file in `../../_planning/adr/`.
If any ADR has `status: assumption` and affects this program → STOP.
Surface to a human. Wait for `accepted`. Then re-check.

**Stop 2 — Spec review:**
Run `{root}/_meta/spec-review.md` on this program.
If verdict is not `OVERALL: PASS` → STOP. Fix blocking items. Re-run. Repeat until PASS.

(Single-program project — no contract dependencies to check unless shared/ exists.)

---

## Task Routing

| Your Task | Load These | Skip These |
|-----------|-----------|------------|
| Add a new command | MANIFEST.md, `../../_planning/adr/` | — |
| Fix a bug | The failing file and its tests | _planning/ (unless fix changes a command interface) |
| Write tests | Code under test | Everything else |
| Change a command interface | `../../_planning/adr/` | src/ until decision is recorded |

---

## Inference Logging
All inferences → `../../_meta/gaps/pending.txt`
Format: `[timestamp] | cli | [file] | inferred "[what]" — no file states this`
