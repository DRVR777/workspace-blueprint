# MANIFEST — _meta/improvement-engine/

## Envelope
| Field | Value |
|-------|-------|
| `id` | meta-improvement-engine |
| `type` | meta |
| `depth` | 2 |
| `parent` | _meta/ |
| `status` | active |

## What I Am
The self-improvement layer. Aggregates telemetry emitted by stage runs,
detects recurring patterns (slow steps, quality failures, routing mismatches),
proposes diffs to the workspace, validates them against historical data, and
either auto-applies low-risk changes or surfaces high-impact changes for human review.

This folder is domain-agnostic. It operates on telemetry and diff objects — never
on the content of any stage's output.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| CONTEXT.md | file | Agent contract: inputs, process, outputs, trigger conditions |
| telemetry-schema.json | file | Schema every telemetry.json must conform to |
| diff-log-schema.json | file | Schema for per-stage diff-log.json entries |

## What I Need From Parent
- Access to all `output/telemetry.json` files across all stage programs
- Write access to `output/diff-log.json` in any stage program

## What I Give To Children
Nothing. This folder has no children.

## What I Return To Parent
- Proposed and applied diffs (referenced in stage diff-log.json files)
- Pattern report (logged to _meta/gaps/pending.txt when a systemic issue is detected)

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Run improvement cycle | CONTEXT.md |
| Understand telemetry format | telemetry-schema.json |
| Understand diff-log format | diff-log-schema.json |

## Layer 0 Test
Every file here must work for Byzantine tax law, protein folding, or jazz harmony.
No domain knowledge may appear in this folder.
