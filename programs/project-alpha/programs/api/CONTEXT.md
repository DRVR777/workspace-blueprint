# programs/api — Task Router

## What This Program Is
[One sentence: what it does, what it exposes, who consumes it.]

---

## Before Writing Any Code

**Stop 1 — Assumption ADR check:**
Read every file in `../../_planning/adr/`.
If any ADR has `status: assumption` and affects this program → STOP.
Surface to a human. Wait for `accepted`. Then re-check.

**Stop 2 — Stub contract check:**
Read every contract listed in this program's MANIFEST.md `External Dependencies` table.
If any contract at `../../shared/contracts/[contract]` has `status: stub` → STOP.
Define the shape before writing code that produces it.

**Stop 3 — Spec review:**
Run `{root}/_meta/spec-review.md` on this program.
If verdict is not `OVERALL: PASS` → STOP. Fix blocking items. Re-run. Repeat until PASS.

---

## Task Routing

| Your Task | Load These | Skip These |
|-----------|-----------|------------|
| Add a new endpoint | MANIFEST.md, relevant contract from `../../shared/contracts/` | Other contracts |
| Fix a bug | The failing source file(s) and tests | All docs |
| Write tests | The feature under test and its contract | — |
| Refactor | Files being refactored, `../../_planning/adr/` | — |
| Update an interface | `../../shared/contracts/[contract]`, `../../_planning/adr/` | src/ until contract is updated |

---

## Hard Rules

1. **Contracts first.** If an endpoint's response shape isn't in `../../shared/contracts/`, define it there before writing code.
2. **No importing from other programs.** Consume `../../shared/` only.
3. **Tests are not optional.** Every endpoint has at minimum one happy-path test.
4. **ADRs are binding.** Check `../../_planning/adr/` before any architectural choice.

---

## Inference Logging
All inferences → `../../_meta/gaps/pending.txt`
Format: `[timestamp] | api | [file] | inferred "[what]" — no file states this`
