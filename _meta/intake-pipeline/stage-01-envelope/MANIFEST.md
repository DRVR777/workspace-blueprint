# MANIFEST — _meta/intake-pipeline/stage-01-envelope/

## Envelope
| Field | Value |
|-------|-------|
| `id` | intake-pipeline-stage-01 |
| `type` | stage |
| `depth` | 3 |
| `parent` | _meta/intake-pipeline/ |
| `status` | active |

## What I Am
Stage 1 of the 5-stage intake pipeline. Wraps any inbound document
with a stable identity (doc-id, hash, timestamp, source metadata).

## What I Need From Parent
Raw document content (text string or file path).

## What I Return To Parent
`_intake/processing/[doc-id]-envelope.json` with `stage: "enveloped"`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Execute this stage | CONTEXT.md |
| Stage complete, proceed | ../stage-02-classify/CONTEXT.md |
