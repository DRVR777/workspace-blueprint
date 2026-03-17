# MANIFEST — {{PROJECT_NAME}}/_meta/

## Envelope
| Field | Value |
|-------|-------|
| `id` | {{PROJECT_NAME}}-meta |
| `type` | meta |
| `depth` | 3 |
| `parent` | programs/{{PROJECT_NAME}}/ |
| `status` | active |

## What I Am
Project-internal gap registry for {{PROJECT_NAME}}.
Inferences made during work on this project land in gaps/pending.txt.
Cross-project inferences escalate to {root}/_meta/gaps/pending.txt.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| gaps/ | folder | Gap registry: pending.txt, CONTEXT.md, gap JSON files |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Log an inference | gaps/pending.txt |
| Find open gaps | gaps/CONTEXT.md |
| Classify inferences | Run {root}/_meta/gap-detection-agent.md with scope: {{PROJECT_NAME}} |
