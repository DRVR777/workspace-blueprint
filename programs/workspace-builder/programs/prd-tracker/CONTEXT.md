# prd-tracker — Task Router

## What This Program Does
Reads the source PRDs and extracts structured requirements.
Maintains the requirements list in `../../_planning/roadmap.md`.
When the PRDs are updated or re-read, this program reconciles the roadmap.

---

## Extraction Process

**Step 1 — Read PRDs:**
Load `../../_planning/prd-source.md` (contains MWP v1, MWP v2, ALWS combined).

**Step 2 — Extract requirements:**
For each PRD section, extract:
- What specific capability or file or pattern is required
- Which PRD and section it comes from
- Whether it's structural (must exist as a file/folder) or behavioral (must be followed as a convention)

**Step 3 — Classify each requirement:**
- `structural` — requires a specific file or folder to exist
- `behavioral` — requires a pattern to be followed (enforced by convention, not file presence)
- `tooling` — requires a script or automation

**Step 4 — Reconcile with roadmap:**
For each extracted requirement:
- If it exists in the roadmap → keep, update notes if needed
- If it's new → add as `❌ not-implemented`
- If it's in roadmap but not in PRD anymore → flag for removal

**Step 5 — Update roadmap:**
Write updated `../../_planning/roadmap.md`.

---

## When to Run
- After reading a new version of the PRDs
- After a major build session (to re-check what was completed)
- Before starting a new build cycle (to confirm the priority order)
