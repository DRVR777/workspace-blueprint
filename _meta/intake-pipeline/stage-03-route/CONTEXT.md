# Stage 03 — Route

## What This Stage Does
Determines exactly where this document belongs and which handler processes it.
Routing is MANIFEST-driven: each hop reads a MANIFEST.md's Routing Rules table.
No route is hard-coded — routes are discovered by traversing MANIFESTs.

## Inputs
`_intake/processing/[doc-id]-envelope.json` with `stage: "classified"`

## Routing Table

| Document Type | Starting Point | Handler |
|---------------|---------------|---------|
| `prd` | programs/MANIFEST.md | `_meta/prd-intake.md` |
| `feature-request` | programs/[project_hint]/MANIFEST.md | `programs/[project]/_planning/CONTEXT.md` |
| `adr` | programs/[project_hint]/_planning/MANIFEST.md | `programs/[project]/_planning/adr/` |
| `contract-update` | programs/[project_hint]/shared/MANIFEST.md | `programs/[project]/shared/contracts/` |
| `question` | — | Not routed. Log to pending.txt. Stop pipeline. |
| `unknown` | — | Route to gap system. Log to pending.txt. Stop pipeline. |

## Process

1. **Read classification from envelope.**

2. **Handle non-routable types immediately.**
   - `question`: Log to `_meta/gaps/pending.txt` as an inference with `requires_human: true`. Update envelope `stage: "stopped"`. Stop.
   - `unknown`: Log to `_meta/gaps/pending.txt`. Update envelope `stage: "stopped"`. Stop.

3. **Resolve project_hint for feature-request, adr, contract-update.**
   If `project_hint` is null:
   - List projects in `programs/` (excluding `_template`)
   - If exactly one project exists: use it
   - If multiple exist: log gap `requires_human: true` and stop
   If `project_hint` names a folder that doesn't exist in `programs/`:
   - For `feature-request`: this may be a new project — reclassify as `prd` and re-route
   - For `adr` and `contract-update`: gap, requires human

4. **Perform hop-by-hop routing.**
   Start at the Starting Point MANIFEST from the Routing Table above.
   For each hop:
   a. Read the MANIFEST.md Routing Rules table
   b. Find the row whose Condition best matches the document
   c. Follow the "Go To" path
   d. Record the hop: `{"from": "[path]", "condition": "[matched condition]", "to": "[path]"}`
   e. Stop when "Go To" is a handler (a .md file), not another folder

5. **Record route in envelope.**
   Update `_intake/processing/[doc-id]-envelope.json`:
   ```json
   {
     ...existing fields...,
     "stage": "routed",
     "route": {
       "handler": "[path to handler file]",
       "hops": [
         {"from": "[path]", "condition": "[text]", "to": "[path]"},
         ...
       ],
       "target_project": "[slug or null]",
       "target_folder": "[final destination folder]"
     }
   }
   ```

## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 4 | Route hops + handler identified | approve / override handler |

## Audit
- [ ] `route.handler` points to a file that exists
- [ ] `route.hops` has at least one entry
- [ ] `target_project` is null (for prd) or names an existing folder in programs/
- [ ] Envelope `stage` updated to `"routed"` (or `"stopped"` for non-routable)

## Outputs
`_intake/processing/[doc-id]-envelope.json` with `stage: "routed"` and `route` object populated.
