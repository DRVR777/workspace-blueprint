# Registry — Agent Contract

## What This Is

Layer 1: the type-aware routing layer. Every task that enters this workspace has a type.
Every type maps to a workspace. This folder is where that mapping lives and evolves.

---

## When to Read This Folder

| Trigger | Step |
|---------|------|
| intake-pipeline stage-02-classify needs to identify a task type | Read REGISTRY.md — scan task_type column |
| intake-pipeline stage-03-route needs to find the workspace path | Read REGISTRY.md — get workspace_path for the matched task_type |
| A task type is not found in REGISTRY.md | Follow Unknown Type Protocol in REGISTRY.md |
| workspace-builder has scaffolded a new project | Write new row to REGISTRY.md |
| A run completes | Update avg_quality, run_count, last_run in REGISTRY.md |

---

## Type Detection Rules

When classifying an incoming task, match against `task_type` values in REGISTRY.md using these signals:

| Signal | Weight |
|--------|--------|
| Document contains `prd`, `product requirements`, `specification` | strong → `prd-intake` |
| Document contains `gap`, `missing`, `inference log`, `pending.txt` content | strong → `gap-detection` |
| Document contains `spec review`, `ADR`, `accepted`, `assumption` | strong → `spec-review` |
| Document is a file path or URL with no envelope | moderate → `document-intake` |
| Document describes a game, simulation, 3D world, engine | strong → `game-engine` |
| Document describes notes, edges, links between concepts | moderate → `knowledge-graph` |

If confidence < 0.7 for all types → `task_type: unknown`, `routing_confidence: 0.0`.

---

## Quality Score Definition

`quality_score` is a float 0.0–1.0 computed from the stage's audit section result:

```
quality_score = (PASS count) / (PASS count + FAIL count)
```

SKIP items are excluded from the calculation.
A run with no audit flags defaults to `quality_score: null` (not included in avg_quality).

---

## Audit

Before closing any registry update, verify:

- [ ] Every updated row has a valid ISO-8601 `last_run` timestamp
- [ ] `avg_quality` was recomputed using the rolling mean formula, not overwritten
- [ ] No rows were deleted (deprecate only)
- [ ] `pending.txt` entry was written for any new registration or unknown type encounter
