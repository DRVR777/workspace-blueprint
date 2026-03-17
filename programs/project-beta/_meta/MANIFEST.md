# MANIFEST — project-beta/_meta/

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-beta-meta |
| `type` | meta |
| `depth` | 3 |
| `parent` | programs/project-beta/ |
| `status` | active |

## What I Am
Project-beta's internal gap registry.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| gaps/ | folder | Gap registry: pending.txt, CONTEXT.md, gap JSON files |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Log an inference | gaps/pending.txt |
| Find open gaps | gaps/CONTEXT.md |
| Classify inferences | Run {root}/_meta/gap-detection-agent.md with scope: project-beta |
