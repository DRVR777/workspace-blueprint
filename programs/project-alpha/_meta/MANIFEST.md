# MANIFEST — project-alpha/_meta/

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-meta |
| `type` | meta |
| `depth` | 3 |
| `parent` | programs/project-alpha/ |
| `status` | active |

## What I Am
Project-alpha's internal gap registry. Inferences made during work on this project
land here. Cross-project inferences escalate to root _meta/gaps/.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| gaps/ | folder | Gap registry: pending.txt, CONTEXT.md, gap JSON files |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Log an inference | gaps/pending.txt |
| Find open gaps | gaps/CONTEXT.md |
| Classify inferences | Run {root}/_meta/gap-detection-agent.md with scope: project-alpha |
