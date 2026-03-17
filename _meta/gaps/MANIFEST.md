# MANIFEST — _meta/gaps/

## Envelope
| Field | Value |
|-------|-------|
| `id` | meta-gaps-registry |
| `type` | gap-registry |
| `depth` | 2 |
| `parent` | _meta/ |
| `status` | active |

## What I Am
The live gap registry. Every open gap in the workspace is tracked here.
pending.txt receives raw inferences. gap-detection-agent.md classifies them.
CONTEXT.md is the ranked index an agent reads to pick the next task.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Ranked open gaps table — the runner reads this |
| pending.txt | file | Inference log — append only, never edit existing entries |
| gap-NNN-[slug].json | files | Individual gap objects (one per gap) |

## What I Need From Parent
gap-schema.json from _meta/ to validate gap object format.

## What I Return To Parent
The next task — always the highest-severity open gap in CONTEXT.md.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Find the next task | CONTEXT.md |
| Log a new inference | pending.txt (append only) |
| Classify inferences into gaps | Run _meta/gap-detection-agent.md |
| Read a specific gap | gap-[NNN]-[slug].json |
