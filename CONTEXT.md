# Workspace — Task Router

## What This Workspace Is

A multi-project workspace. Each project is a fully self-contained folder with its own
architecture decisions, programs, contracts, and gap registry.
Root `_meta/` is the only shared infrastructure.

---

## Task Routing

| Your Task | Go To | Also Load |
|-----------|-------|-----------|
| Start or end any session | `_meta/runner.md` | `_meta/gaps/CONTEXT.md` |
| Introduce a new project from a PRD | `_intake/queue/` → `_meta/prd-intake.md` | — |
| Work inside an existing project | `programs/[project]/CONTEXT.md` | `programs/[project]/MANIFEST.md` |
| Look up an architectural pattern | `_core/CONVENTIONS.md` | — |
| Onboard a new workspace domain | `setup/questionnaire.md` | — |
| Run spec review on a program | `_meta/spec-review.md` | Program's MANIFEST.md + CONTEXT.md |
| Understand a status transition gate | `_meta/status-transitions.md` | — |
| Detect or classify gaps | `_meta/gap-detection-agent.md` | `_meta/gaps/pending.txt` |
| Find the highest-priority open gap | `_meta/gaps/CONTEXT.md` | — |
| Generate an execution prompt | `_meta/ur-prompt.md` | The gap object being addressed |
| Orient with no prior context | Root `MANIFEST.md` → here | — |

---

## Active Projects

| Project | Purpose | Status |
|---------|---------|--------|
| `programs/workspace-builder/` | Self-improvement engine — tracks and builds PRD requirements | active |
| `programs/knowledge-graph/` | Cognitive Document System — self-navigating 5D document graph | specced |
| `programs/game_engine/` | NEXUS — spatial computing substrate for infinite persistent multiplayer 3D worlds | scaffold |
| `programs/ELEV8/` | Dreamworld hackathon prototype — failure analysis reference | reference |
| `programs/dreamworld/` | Dreamworld vision PRD — requirements source for game_engine | reference |
| `programs/project-alpha/` | Example placeholder — replace with first real project | scaffold |
| `programs/project-beta/` | Example placeholder — replace with second real project | scaffold |

Add a row here when a new project is scaffolded via `intake:` or `_meta/prd-intake.md`. Remove when archived.

---

## Workspace Rules

1. One project per folder. Projects never share source code.
2. Cross-project contracts live in `_meta/contracts/` — not inside either project.
3. An agent inside one project never loads another project's files.
4. `_meta/` at root is the only shared infrastructure.
5. When errors or inconsistencies are found — fix them. Do not surface them as questions.

---

## Cross-Project Gap Flow

```
[project]/_meta/gaps/pending.txt   ← project-internal inferences
root _meta/gaps/pending.txt        ← cross-project inferences
        ↓
_meta/gap-detection-agent.md       ← classifies entries into formal gap objects
        ↓
_meta/gaps/CONTEXT.md              ← ranked open gaps → next session's starting point
```
