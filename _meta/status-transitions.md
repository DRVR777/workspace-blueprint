# Status Transitions

## What This Is

The gate rules. Every status change for a project or program has explicit criteria.
Nothing changes status without meeting the criteria.
Criteria are not suggestions — they are the definition of what the status means.

---

## Status Model

```
PROGRAM:  scaffold → specced → active → complete
PROJECT:  scaffold → active  → complete
```

---

## Program Transitions

### scaffold → specced

**What it means:** The plan is validated. Building can begin.

**Criteria — all must be true:**
1. All in-scope ADRs have status `accepted` (none are `assumption` or `proposed`)
2. All contracts in the program's External Dependencies have defined shapes (none are `stub`)
3. Spec review verdict from `_meta/spec-review.md` is OVERALL: PASS
4. No blocking gaps in `[project]/_meta/gaps/CONTEXT.md` affect this program

**How to trigger:**
Run `_meta/spec-review.md` on this program.
If verdict is PASS: change `status` field in program's `MANIFEST.md` from `scaffold` to `specced`.
Log the transition to `[project]/_meta/gaps/pending.txt`.

**What stays the same:**
The CONTEXT.md does not change on this transition.
The ADRs do not change.
Only the MANIFEST.md `status` field changes.

---

### specced → active

**What it means:** Building has started. Code exists.

**Criteria:**
1. Program status is `specced`
2. At least one file exists in `src/`

**How to trigger:**
Automatic — when the first file is written to `src/`, status moves to `active`.
Update `status` in MANIFEST.md when this happens.

---

### active → complete

**What it means:** The program is done. All contracts it produces are fulfilled.

**Criteria — all must be true:**
1. All acceptance criteria from the program's contracts are met
2. All tests pass
3. No blocking gaps affecting this program remain open
4. The program's outputs are in the expected location

**How to trigger:**
Run `_meta/spec-review.md` again at completion (second pass — verifies implementation matches plan).
If PASS: change status to `complete`. Update project MANIFEST.md.

---

## Project Transitions

### scaffold → active

**What it means:** At least one program is building.

**Criteria:**
1. At least one program in `programs/` has status `active` or `specced`

**How to trigger:**
Automatic — when the first program reaches `specced`, update project MANIFEST.md status to `active`.

---

### active → complete

**What it means:** The project is done.

**Criteria:**
1. All programs in `programs/` have status `complete`
2. All contracts in `shared/contracts/` are fulfilled
3. No open gaps remain in `[project]/_meta/gaps/CONTEXT.md`

**How to trigger:**
Manual — a human or agent confirms all criteria are met and updates project MANIFEST.md.

---

## The Build Order Rule

Programs build in dependency order. A program cannot move to `specced` if any
contract it *consumes* is produced by a program still at `scaffold`.

Determine build order from the dependency graph:
1. Read every program's MANIFEST.md `External Dependencies` table
2. A program with no external dependencies builds first
3. A program that consumes another program's contract waits until that contract is defined

This order must be documented in `_planning/roadmap.md` before any program moves to `specced`.

---

## Status Reference

| Status | Meaning | Blocked Until |
|--------|---------|---------------|
| `scaffold` | Structure exists, nothing validated | spec-review PASS |
| `specced` | Plan validated, ready to build | first src/ file written |
| `active` | Build in progress | all acceptance criteria met + tests pass |
| `complete` | Done | — |

---

## Hard Rules

1. **Never skip a status.** scaffold → active is not allowed. scaffold → specced → active.
2. **spec-review.md is the only path to specced.** No other gate. No exceptions.
3. **An assumption ADR blocks all downstream programs.** If ADR-003 is `assumption` and programs A and B depend on it, neither A nor B can move to `specced` until ADR-003 is `accepted`.
4. **A stub contract blocks all consumers.** If contract X is `stub`, every program that consumes X is blocked at `scaffold`.
5. **Status is always the MANIFEST.md `status` field.** Not a separate tracker, not a spreadsheet — the MANIFEST is the record.
