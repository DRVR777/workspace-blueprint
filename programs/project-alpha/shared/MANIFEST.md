# MANIFEST — project-alpha/shared

## Envelope
| Field | Value |
|-------|-------|
| `id` | project-alpha-shared |
| `type` | contracts |
| `depth` | 3 |
| `parent` | programs/project-alpha/ |
| `status` | active |

## What I Am
The hard boundary between programs. Programs never import from each other.
If two programs need to agree on a data shape — the definition lives here.

## The Rule
If it's not defined here, both sides will infer independently → divergence → missing_bridge gap.

## Contracts Needed

| Contract | Produced By | Consumed By | Status |
|----------|-------------|-------------|--------|
| [define from PRD] | [program] | [program] | stub |
