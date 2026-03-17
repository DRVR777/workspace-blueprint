# MANIFEST — _meta/intake-pipeline/

## Envelope
| Field | Value |
|-------|-------|
| `id` | meta-intake-pipeline |
| `type` | pipeline |
| `depth` | 2 |
| `parent` | _meta/ |
| `status` | active |

## What I Am
The 5-stage pipeline for processing all inbound documents.
Every document entering this workspace — PRD, feature request, ADR, contract update —
flows through these five stages before being placed in the correct location.

Stage handoffs: each stage reads the previous stage's output/ folder.
The pipeline is invoked by prd-intake.md (which wraps Stage 4 for PRD-type documents).

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| stage-01-envelope/ | folder | Wrap document with ID, hash, timestamp, source metadata |
| stage-02-classify/ | folder | Detect document type with confidence score |
| stage-03-route/ | folder | MANIFEST-driven hop-by-hop routing decision |
| stage-04-place/ | folder | Execute routing: call the handler, update MANIFESTs |
| stage-05-update/ | folder | Log telemetry, archive envelope, update registry |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| New document arrives (any type) | stage-01-envelope/CONTEXT.md |
| Document already enveloped | stage-02-classify/CONTEXT.md |
| Document already classified | stage-03-route/CONTEXT.md |
| Route already determined | stage-04-place/CONTEXT.md |
| Document already placed | stage-05-update/CONTEXT.md |

## Pipeline Contract
Input: raw document text or file path
Output: document placed at correct location + all MANIFESTs updated + envelope archived
