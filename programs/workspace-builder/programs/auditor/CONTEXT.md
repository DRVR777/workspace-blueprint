# auditor — Task Router

## What This Program Does
Audits the current workspace state against PRD requirements.
Reads `_planning/roadmap.md` to find unimplemented requirements,
checks whether they exist in the workspace, and generates gap objects.

---

## Audit Process

For each row in `../../_planning/roadmap.md` where Status = `❌` or `⚠️`:

**Step 1 — Locate what would satisfy this requirement:**
Read the requirement. Identify what file(s) or folder(s) would implement it.
Check if those files exist and contain the expected content.

**Step 2 — Classify the gap:**
- File exists but is empty/stub → `shallow_node`
- File exists but contradicts requirement → `shallow_node`
- File doesn't exist → `missing_composition`
- Two files exist but aren't connected when they should be → `missing_bridge`

**Step 3 — Write gap entry:**
Append to `../../_meta/gaps/pending.txt`:
```
[timestamp] | workspace-builder/auditor | roadmap.md | inferred "[what is missing]" — PRD requires this, no file implements it
```

**Step 4 — Update roadmap:**
Change status to `⚠️` if partially implemented, keep `❌` if missing entirely.
Add a note in the Notes column about what was found.

**Step 5 — Report:**
Print a summary: N requirements checked, M gaps found, severity breakdown.

---

## Audit Scope

Run against all three PRD source documents:
- `../../_planning/prd-source.md` (MWP v1, MWP v2, ALWS combined)
- Current workspace file tree (read via MANIFEST chain from root)

Do NOT read programs' src/ folders — audit is structural, not code-level.

---

## Output

After audit, the gaps in `../../_meta/gaps/pending.txt` feed into:
`{root}/_meta/gap-detection-agent.md` → formal gap objects → `{root}/_meta/runner.md`
