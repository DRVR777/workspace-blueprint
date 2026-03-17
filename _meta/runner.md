# Runner — Session Protocol

## What This Is

The outer loop. How to start any session, what to do during it, and how to end it.
This is the protocol that keeps the system moving between sessions without losing state.

---

## Starting a Session

Read these in order before doing anything else:

1. **Root `MANIFEST.md`** — what projects exist, their status (scaffold/active)
2. **Root `_meta/gaps/CONTEXT.md`** — open cross-project gaps, sorted by severity
3. **`[project]/_meta/gaps/CONTEXT.md`** — if you're working on a specific project

Pick your starting task:
- If there are blocking gaps → close the highest-severity blocking gap first
- If no blocking gaps → pick the highest-severity degrading gap
- If no open gaps → check `_planning/roadmap.md` for next build task

**Before picking a build task:** check whether any program in the target project is at `scaffold`
with all its ADRs `accepted` and all its contracts non-stub.
If yes → that program is ready for spec review. Run `_meta/spec-review.md` on it before writing code.
A program cannot move to `active` without a spec-review PASS. See `_meta/status-transitions.md`.

Feed the gap (or spec-review result) to `_meta/ur-prompt.md` Steps 1–7 to generate an execution prompt.
Execute that prompt. That is the session.

---

## During a Session

Every inference → `pending.txt` immediately. Not at the end. During.

```
[ISO-8601-timestamp] | [location] | [file-consulted] | inferred "[what]" — no file states this
```

Scope rules:
- Inference about something inside one project → `[project]/_meta/gaps/pending.txt`
- Inference about something crossing project boundaries → root `_meta/gaps/pending.txt`

---

## Ending a Session

Run these in order before closing:

1. Confirm all inferences are logged to the correct `pending.txt`
2. **Run `python _meta/scripts/validate_manifests.py`** — catches any folders created this session without MANIFEST.md. Fix all ERRORs before closing.
3. **Run `_meta/guards/guard-05-manifest-reconcile.md`** — catches stale MANIFESTs (missing or ghost entries). Auto-fix additions; log the rest.
4. Run `_meta/gap-detection-agent.md` with the correct scope:
   - `scope: [project-name]` for project-internal work
   - `scope: root` for cross-project work
5. Update `status` field on any gap objects closed during this session
6. Update the gap table in the relevant `_meta/gaps/CONTEXT.md`
7. The highest-severity open gap after detection = next session's starting point

---

## The One Rule

If you don't know what to do next: read `_meta/gaps/CONTEXT.md`.

The system always has a next task. It is always the highest-severity open gap.
If there are no open gaps, the system is done — which means either the project is
complete or the gap detection didn't run properly.

---

## Agent Behavior Rule

When you encounter broken references, stale content, missing files, or schema mismatches
during any session — fix them immediately. Do not surface them as questions.
Finding a fault and fixing it are one task, not two.

Log a `[timestamp] | [location] | [file] | inferred "fixed [what]"` entry to the
appropriate pending.txt so the repair is traceable.

---

## Special Cases

| Situation | What to do |
|-----------|-----------|
| New document received (any type) | Run `_meta/intake-pipeline/` stages 01→05 |
| PRD received (shortcut) | Run `_meta/prd-intake.md` directly (skips pipeline) |
| Project exists but PRD is an update | Check project status — if active, do not re-scaffold |
| Blocking gap with no clear resolution | Escalate: write an ADR in `_planning/adr/` with status `proposed`, surface to human |
| Two gaps at equal severity | Pick the one closer to the root (blocking a more fundamental layer) |
| Program at scaffold, ADRs accepted, contracts defined | Run `_meta/spec-review.md` — do not write code until OVERALL: PASS |
| spec-review returns FAIL | Fix blocking items listed in verdict — do not change program status |
| spec-review returns PASS | Change program MANIFEST.md status scaffold → specced, then pick first build task |
