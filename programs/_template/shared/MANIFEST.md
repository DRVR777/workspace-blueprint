# MANIFEST — {{PROJECT_NAME}}/shared

## Envelope
| Field | Value |
|-------|-------|
| `id` | {{PROJECT_NAME}}-shared |
| `type` | contracts |
| `depth` | 3 |
| `parent` | programs/{{PROJECT_NAME}}/ |
| `status` | active |

## What I Am
The hard boundary between programs. Programs never import from each other.
If two programs need to agree on a data shape — the definition lives here.

## The Rule
If it's not defined here, both sides will infer independently → divergence → missing_bridge gap.

## What I Contain

| Name | Type | Purpose |
|------|------|---------|
| voice.md | file | Voice rules as error conditions — Hard Constraints, Sentence Rules, Pacing (P-12) |
| contracts/ | folder | Data shape contracts between programs |

## Contracts

| Contract | Produced By | Consumed By | Status |
|----------|-------------|-------------|--------|
| [fill from PRD] | [program] | [program] | stub |

## What "stub" means
A stub contract is a placeholder. No program may build against a stub.
Define the shape in the contract file before either program starts building.
