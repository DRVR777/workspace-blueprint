# MANIFEST — production/docs

## Envelope
| Field | Value |
|-------|-------|
| `id` | production-docs |
| `type` | reference |
| `depth` | 2 |
| `parent` | production/ |
| `status` | active |

## What I Am
Technical reference docs for build agents. Load per stage, not all at once.
These files define the quality floor — minimum standards, available components,
visual specifications. They do not change during a run.

## What I Contain
| File | Load When | Skip When |
|------|-----------|-----------|
| tech-standards.md | Stage 02 (spec writing), any quality check | Stage 04 output formatting only |
| component-library.md | Stage 03 (build) — check what exists before building from scratch | Stage 02 (planning only) |
| design-system.md | Stage 03 (build) — visual standards and color tokens | Stage 02 (planning only) |

## What I Need From Parent
Nothing — these are terminal reference files.

## What I Give To Children
Nothing — these are read-only reference files.

## What I Return To Parent
Technical, component, and visual specifications that constrain build agent output.

## Routing Rules
Stage 02 agents: load tech-standards.md only.
Stage 03 agents: load all three.
Stage 04 agents: load none unless running a specific quality verification check.
Never load all three unless you are at Stage 03.

## Gap Status
No open gaps.
