# workspace-builder — Task Router

## What This Project Is
The workspace's self-improvement engine. It tracks what the source PRDs require,
audits what currently exists, and generates build tasks for everything missing.

**Work target:** This project builds things in the workspace root (`../..`), not inside itself.
Files it creates land there. The only files it writes to inside `programs/workspace-builder/` are
`_planning/roadmap.md` (status) and `_meta/gaps/pending.txt` (gap log).

`{root}` throughout this file = `C:\Users\Quandale Dingle\yearTwo777\workspace-blueprint\workspace-blueprint\`

---

## Current State (as of 2026-03-14)

**All 10 build priorities complete.** The workspace-builder has fully implemented itself.

This means: when loaded with no specific task, there is nothing left to build from the original PRDs.
The correct action is one of:
- **A new PRD has arrived** → run `programs/prd-tracker/CONTEXT.md` to extract new requirements, then audit
- **A specific gap was logged** → read `_meta/gaps/pending.txt`, find highest-severity open entry, run build loop on it
- **A workspace audit was requested** → run `programs/auditor/CONTEXT.md` against the current roadmap ⚠️ items

Two requirements remain at ⚠️ (partial) — both are accepted gaps, not open work:
- `Fractal MANIFEST at every folder` → partial coverage is sufficient; deep sub-folders covered on-demand by P-25 (new folder protocol)
- `_core/templates/ folder` → `programs/_template/` serves this role; a separate `_core/templates/` is not needed

Do NOT start building things that don't exist in the PRD. workspace-builder only acts on explicit requirements.

---

## Task Routing

| Your Task | Go To | Also Load |
|-----------|-------|-----------|
| See all PRD requirements and status | `_planning/roadmap.md` | `_planning/prd-source.md` |
| Run a full audit pass | `programs/auditor/CONTEXT.md` | `_planning/roadmap.md` |
| Extract new requirements from PRD | `programs/prd-tracker/CONTEXT.md` | `_planning/prd-source.md` |
| Execute a specific build task | `../../_meta/runner.md` | The relevant gap object |
| Log a gap found during audit | `_meta/gaps/pending.txt` | — |

---

## The Build Loop

When a new requirement exists (Status = ❌ or ⚠️ in roadmap):
1. Read `_planning/roadmap.md` — find the first row where Status = `❌` or `⚠️`
2. Route to `programs/auditor/CONTEXT.md` for that requirement
3. Auditor generates a gap object
4. Feed gap to `../../_meta/runner.md`
5. Runner executes and closes the gap
6. Update `_planning/roadmap.md` row to `✅`
7. Return to step 1

**If roadmap shows all ✅:** the build loop is idle. Do not invent work. Wait for new PRDs or explicit gaps.

---

## Hard Rule
This project modifies the workspace. Every change it makes must be logged to
`_meta/gaps/pending.txt` as an inference entry before the change is committed.
