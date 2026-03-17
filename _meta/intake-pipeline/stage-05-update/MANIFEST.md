# MANIFEST — _meta/intake-pipeline/stage-05-update/

## Envelope
| Field | Value |
|-------|-------|
| `id` | intake-pipeline-stage-05 |
| `type` | stage |
| `depth` | 3 |
| `parent` | _meta/intake-pipeline/ |
| `status` | active |

## What I Am
Stage 5 of the 5-stage intake pipeline. Closes the loop: archives the envelope,
moves source file to processed/, and logs the intake event.

## What I Need From Parent
`_intake/processing/[doc-id]-envelope.json` with `stage: "placed"`

## What I Return To Parent
- `_intake/processed/[doc-id]-envelope.json` (envelope archived)
- `_intake/intake-log.md` (intake event recorded)
- Root `_meta/gaps/pending.txt` (intake inference logged)
- `stage: "complete"` on the envelope (in processed/)

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Execute this stage | CONTEXT.md |
| Pipeline complete | Intake done — return to caller |
