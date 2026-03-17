# Stage 04 — Place

## What This Stage Does
Executes the route. Calls the handler with the document content.
Updates every MANIFEST.md touched by the placement.
This is the stage where files actually get created.

## Inputs
`_intake/processing/[doc-id]-envelope.json` with `stage: "routed"`

## Process

1. **Read `raw_content` and `route` from envelope.**

2. **Call the handler.**
   Read the handler file identified in `route.handler`.
   Pass `raw_content` as the document input.
   Execute the handler's full process.

   Handler dispatch table:
   | Handler | What it does |
   |---------|-------------|
   | `_meta/prd-intake.md` | Full PRD scaffold — creates project, ADRs, contracts, programs |
   | `_planning/CONTEXT.md` | Opens a gap or feature planning doc in the target project |
   | `_planning/adr/` | Creates a new ADR file in the target project |
   | `shared/contracts/` | Creates or updates a contract file |

3. **Collect all files and folders created by the handler.**
   Keep a list: `placed_files = [list of absolute paths created or modified]`

4. **Update all MANIFESTs touched.**
   For each file/folder in `placed_files`:
   - Check parent MANIFEST.md "What I Contain" table
   - If the item is not listed: add a row
   - If the item IS listed but status has changed: update the row
   Use the same logic as `scaffold_manifest.py --update-parent`.

5. **Run MANIFEST validation on placed files.**
   For each new folder in `placed_files`:
   - Verify it has MANIFEST.md
   - If not: create a stub via the scaffold_manifest logic
   This enforces the New Folder Protocol without agent effort.

6. **Update envelope.**
   ```json
   {
     ...existing fields...,
     "stage": "placed",
     "placed_at": "[ISO-8601 timestamp]",
     "placed_files": ["list of created/modified paths"]
   }
   ```

## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 2 (after handler completes) | Summary of files created | approve / flag issues |
| Step 4 | MANIFEST updates made | approve |

## Audit
- [ ] Handler completed without errors
- [ ] Every new folder has a MANIFEST.md
- [ ] Every created file is listed in its parent MANIFEST.md "What I Contain"
- [ ] `placed_files` list matches actual files on disk
- [ ] No files placed outside the `target_folder` path (no scope leak)
- [ ] Envelope `stage` updated to `"placed"`

## Outputs
- All files created by the handler, at their correct locations
- All parent MANIFESTs updated
- `_intake/processing/[doc-id]-envelope.json` with `stage: "placed"`
