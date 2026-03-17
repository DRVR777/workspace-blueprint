# Workspace Registry

**Source of truth for task type → workspace routing.**
Updated by: intake-pipeline stage-05-update (quality), workspace-builder (new registrations).
Read by: intake-pipeline stage-02-classify, stage-03-route.

---

## Registered Workspaces

| task_type | workspace_path | status | avg_quality | run_count | last_run |
|-----------|---------------|--------|-------------|-----------|----------|
| `prd-intake` | `_meta/prd-intake.md` | active | — | 0 | — |
| `gap-detection` | `_meta/gap-detection-agent.md` | active | — | 0 | — |
| `spec-review` | `_meta/spec-review.md` | active | — | 0 | — |
| `document-intake` | `_meta/intake-pipeline/` | active | — | 0 | — |
| `writing` | `programs/workspace-builder/` | scaffold | — | 0 | — |
| `knowledge-graph` | `programs/knowledge-graph/` | specced | — | 0 | — |
| `game-engine` | `programs/game_engine/` | specced | — | 0 | — |
| `oracle` | `programs/oracle/` | specced | — | 0 | — |

**avg_quality**: float 0.0–1.0. Mean of `quality_score` across all telemetry entries for this type. `—` = no runs yet.
**run_count**: total completed runs of this workspace type.
**last_run**: ISO-8601 UTC timestamp of most recent completed run.

---

## Unknown Type Protocol

When intake-pipeline stage-02-classify encounters a `task_type` not in this table:

1. Set `routing_confidence: 0.0` in the envelope
2. Write to `_meta/gaps/pending.txt`:
   `[timestamp] | _registry/REGISTRY.md | stage-02-classify | inferred "unknown task_type: [type] — workspace-builder pipeline needed"`
3. Stage-03-route halts and surfaces to human
4. Human either:
   - **A)** Points to an existing workspace → add row to this table manually, re-run route
   - **B)** Confirms new workspace needed → type `intake: "[task description]"` to trigger workspace-builder

---

## Quality Update Protocol

After every completed run, stage-05-update writes one entry to `_meta/improvement-engine/` telemetry,
then updates this table:

1. `run_count` += 1
2. `last_run` = current ISO-8601 UTC timestamp
3. `avg_quality` = rolling mean: `((avg_quality × (run_count - 1)) + new_quality_score) / run_count`
   Round to 2 decimal places.

---

## New Registration Protocol

When workspace-builder scaffolds a new workspace:

1. Add a row with `status: scaffold`, `avg_quality: —`, `run_count: 0`, `last_run: —`
2. Log to `_meta/gaps/pending.txt`:
   `[timestamp] | _registry/REGISTRY.md | workspace-builder | registered new workspace type "[task_type]"`
3. When the workspace reaches `status: active`, update the row's status field

---

## REGISTRY ↔ MANIFEST Sync Rule

REGISTRY status must mirror each project's `programs/[project]/MANIFEST.md` `status` field.
MANIFEST is canonical. REGISTRY is a routing index — it must never diverge from MANIFEST.
At session close, the runner updates REGISTRY status for any project whose MANIFEST.md status changed.

---

## Deprecation Protocol

To deprecate a workspace type:
1. Change `status` to `deprecated`
2. Add a `replaced_by` note in the row (use the Notes column if needed)
3. Never delete rows — keep the full history
