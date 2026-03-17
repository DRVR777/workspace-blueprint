# MANIFEST — _meta/intake-pipeline/stage-04-place/

## Envelope
| Field | Value |
|-------|-------|
| `id` | intake-pipeline-stage-04 |
| `type` | stage |
| `depth` | 3 |
| `parent` | _meta/intake-pipeline/ |
| `status` | active |

## What I Am
Stage 4 of the 5-stage intake pipeline. Executes the route: calls the handler,
creates files, and ensures every new folder and file is reflected in MANIFEST.md files.

## What I Need From Parent
`_intake/processing/[doc-id]-envelope.json` with `stage: "routed"`

## What I Return To Parent
- All files created by the handler at their target locations
- All parent MANIFESTs updated
- Envelope updated with `placed_files` list and `stage: "placed"`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Execute this stage | CONTEXT.md |
| Handler is prd-intake.md | {root}/_meta/prd-intake.md |
| Stage complete, proceed | ../stage-05-update/CONTEXT.md |
