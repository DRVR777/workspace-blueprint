<!-- See {root}/_meta/ur-prompt.md before reading this file. -->
<!-- {root} = C:\Users\Quandale Dingle\yearTwo777\workspace-blueprint\workspace-blueprint\ -->
# workspace-builder — Project Map

## Hard Rule
Depth 1 only. Program names and one-line purposes.

---

## Programs

| Program | Purpose | Status |
|---------|---------|--------|
| `programs/prd-tracker/` | Extracts requirements from source PRDs, tracks implementation status | active |
| `programs/auditor/` | Audits current workspace files against PRD requirements, generates gaps | active |

---

## How This Project Works

The workspace-builder runs a recursive loop:

```
1. prd-tracker reads _planning/prd-source.md
   → produces: requirements list with implemented/not-implemented status

2. auditor reads requirements list + current workspace files
   → produces: gap objects for unimplemented requirements

3. Gap objects feed into ../../_meta/runner.md
   → runner executes gap closure on the workspace

4. After execution: prd-tracker re-audits
   → cycle repeats until all requirements are implemented
```

## Navigation

| You want to... | Go to |
|----------------|-------|
| See PRD requirements | _planning/prd-source.md |
| Check what's implemented | _planning/roadmap.md |
| Run an audit | programs/auditor/CONTEXT.md |
| Start the build loop | ../../_meta/runner.md |
