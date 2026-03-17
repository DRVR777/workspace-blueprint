# MANIFEST — programs/

## Envelope
| Field | Value |
|-------|-------|
| `id` | workspace-programs |
| `type` | programs-registry |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | active |

## What I Am
The workspace's project registry. Every project lives here.
`_template/` is the canonical empty clone source — never modify it directly.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| `_template/` | folder | active | Empty project scaffold — clone source, never populated directly |
| `workspace-builder/` | folder | active | Self-improvement project — tracks PRD requirements, builds missing pieces |
| `knowledge-graph/` | folder | specced | Cognitive Document System — self-navigating 5D document graph |
| `game_engine/` | folder | scaffold | NEXUS — spatial computing substrate for infinite persistent 3D worlds |
| `ELEV8/` | folder | reference | Dreamworld hackathon prototype — reference for failure analysis |
| `dreamworld/` | folder | reference | Dreamworld vision PRD — requirements source for game_engine |
| `project-alpha/` | folder | scaffold | Example placeholder — replace with your first real project |
| `project-beta/` | folder | scaffold | Example placeholder — replace with your second real project |

## The Template Rule
`_template/` is never edited by hand or by agents.
It is cloned by `{root}/_meta/scripts/new_project.py` whenever a new project is created.
After cloning, the clone is populated. The template stays empty and clean.

## Adding a New Project
Two paths:
1. **Inline**: type `intake: "[prd text]"` in any conversation — agent runs `{root}/_meta/prd-intake.md`
2. **File drop**: put a PRD file in `{root}/_intake/queue/` — same agent runs automatically
3. **Script**: `python {root}/_meta/scripts/new_project.py <project-name> --prd @file.md`

All three paths clone `_template/`, populate from the PRD, and leave `_template/` untouched.

## Rules
1. Projects never import from each other directly.
2. Cross-project interfaces go in `{root}/_meta/contracts/` (not inside any project).
3. `_template/` contains `{{PROJECT_NAME}}` placeholders — do not replace them manually.
