# MANIFEST — _meta/intake-pipeline/stage-03-route/

## Envelope
| Field | Value |
|-------|-------|
| `id` | intake-pipeline-stage-03 |
| `type` | stage |
| `depth` | 3 |
| `parent` | _meta/intake-pipeline/ |
| `status` | active |

## What I Am
Stage 3 of the 5-stage intake pipeline. MANIFEST-driven hop-by-hop routing.
Determines the correct handler and target folder for each document type.

## What I Need From Parent
`_intake/processing/[doc-id]-envelope.json` with `stage: "classified"`

## What I Return To Parent
Same envelope file updated with `route` object and `stage: "routed"`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Execute this stage | CONTEXT.md |
| Stage complete, proceed | ../stage-04-place/CONTEXT.md |
| Document type is question or unknown | Stop — log to _meta/gaps/pending.txt |
