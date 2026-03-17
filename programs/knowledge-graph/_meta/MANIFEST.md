# MANIFEST — knowledge-graph/_meta/

## Envelope
| Field | Value |
|-------|-------|
| `id` | knowledge-graph-meta |
| `type` | meta |
| `depth` | 3 |
| `parent` | programs/knowledge-graph/ |
| `status` | active |

## What I Am
Internal gap registry for the knowledge-graph project.
Tracks assumption ADRs that need human validation before building starts.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| gaps/ | folder | Gap registry: pending.txt, CONTEXT.md |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Log an inference | gaps/pending.txt |
| Find open gaps | gaps/CONTEXT.md |
| Classify inferences | Run {root}/_meta/gap-detection-agent.md with scope: knowledge-graph |
