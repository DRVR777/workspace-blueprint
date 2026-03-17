# MANIFEST — _meta/intake-pipeline/stage-02-classify/

## Envelope
| Field | Value |
|-------|-------|
| `id` | intake-pipeline-stage-02 |
| `type` | stage |
| `depth` | 3 |
| `parent` | _meta/intake-pipeline/ |
| `status` | active |

## What I Am
Stage 2 of the 5-stage intake pipeline. Classifies document type
(prd, feature-request, adr, contract-update, question, unknown) with confidence score.

## What I Need From Parent
`_intake/processing/[doc-id]-envelope.json` with `stage: "enveloped"`

## What I Return To Parent
Same envelope file updated with `classification` object and `stage: "classified"`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Execute this stage | CONTEXT.md |
| Stage complete, proceed | ../stage-03-route/CONTEXT.md |
