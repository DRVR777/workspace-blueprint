<!-- See _meta/ur-prompt.md before reading this file. -->
# Workspace Map

## Hard Rule
This file maps depth 1 only. Project names and one-line purposes. Nothing else.
All project internals live in the project's own CLAUDE.md and CONTEXT.md.
If a folder's internal structure appears here, remove it.

## Agent Session Start (REQUIRED — do this before any work)
1. Read `_bus/broadcast.md` (last 40 lines) — shared agent timeline
2. Read `_bus/agents/[your-agent-name]/inbox.md` — your messages
3. Read `_bus/convention_violations.md` — fix any in your domain first
4. Update `_bus/agents/[your-agent-name]/status.md` with your current state

---

## Projects

All projects live in `programs/`. See `programs/MANIFEST.md`.

| Project | Purpose | Status |
|---------|---------|--------|
| `programs/_template/` | Empty clone source — never populate directly | template |
| `programs/workspace-builder/` | Self-improvement engine — tracks PRD requirements, builds missing pieces | active |
| `programs/knowledge-graph/` | Cognitive Document System — self-navigating 5D document graph | specced |
| `programs/game_engine/` | NEXUS — spatial computing substrate for infinite persistent multiplayer 3D worlds | scaffold |
| `programs/oracle/` | ORACLE — AI-native trading intelligence platform for Polymarket and Solana; signal ingestion, whale detection, OSINT fusion, multi-pass reasoning, autonomous execution, self-improving post-mortem knowledge base | scaffold |
| `programs/ELEV8/` | Dreamworld hackathon prototype — reference for failure analysis | reference |
| `programs/dreamworld/` | Dreamworld vision PRD — requirements source for game_engine | reference |
| `programs/project-alpha/` | Example placeholder — replace with your first real project | scaffold |
| `programs/project-beta/` | Example placeholder — replace with your second real project | scaffold |
| `_examples/` | Teaching template: Acme DevRel content system | reference |

**Deprecated (root level):** `project-alpha/` and `project-beta/` at workspace root are superseded
by `programs/project-alpha/` and `programs/project-beta/`. Ignore the root-level copies.

---

## Cross-Project Dependencies

Documented in `_meta/gaps/` as `missing_bridge` gaps with `scope: root`.
Never document cross-project dependencies inside a project's own files.

---

## Navigation

| You want to... | Go to |
|----------------|-------|
| Work inside a project | `programs/[project]/CONTEXT.md` |
| Understand a project's structure | `programs/[project]/CLAUDE.md` |
| Plan a new project | `programs/[project]/_planning/CONTEXT.md` |
| Create a project from a PRD (inline) | Type `intake: "[prd text]"` — agent runs `_meta/prd-intake.md` |
| Create a project from a PRD (file) | Drop PRD in `_intake/queue/`, then run `_meta/prd-intake.md` |
| Create a project from a PRD (script) | `python _meta/scripts/new_project.py <name> --prd @file.md` |
| Start or end a session | `_meta/runner.md` |
| Surface a cross-project dependency | `_meta/gaps/pending.txt` |
| See workspace build status | `programs/workspace-builder/_planning/roadmap.md` |
| Look up any architectural pattern | `_core/CONVENTIONS.md` |
| Route a task to the right workspace | `_registry/REGISTRY.md` |
| Orient with no prior context | `MANIFEST.md`, then here |
| Pick up a previous session | `leftOffHere.md` — read this first, nothing else |
| Close a session / log progress | Run `_meta/session-close.md` protocol → updates `leftOffHere.md` |

## Trigger Keywords
- `intake: "[prd text]"` — treat as PRD, run `_meta/prd-intake.md` with the provided text
- `run gaps` — run `_meta/runner.md` to process pending inferences and close gaps
- `audit` — run `programs/workspace-builder/programs/auditor/CONTEXT.md` to check PRD coverage
- `setup` — run `setup/questionnaire.md` to onboard a new workspace domain and generate voice rules
- `status` — scan all `programs/*/programs/*/output/` folders and report what has been produced vs what is still empty
- `wrap up` — run `_meta/session-close.md` protocol, overwrite `leftOffHere.md` with current session state
- `improve` — run `_meta/improvement-engine/CONTEXT.md` to collect telemetry, detect patterns, and apply or propose diffs
- `graph` — run `_meta/graph-engine/CONTEXT.md` to rebuild graph.json and detect orphan/broken-edge gaps

---

## Workspace Rules

1. One project per folder. Projects do not share source code.
2. Shared contracts between projects live in `_meta/contracts/` — not inside either project.
3. An agent working in one project never loads another project's docs.
4. `_meta/` at root is the only shared infrastructure.
5. Each project has its own `_meta/` for project-internal gaps.
6. **Fix-first rule:** When an agent identifies errors, broken references, stale content,
   or missing files — fix them immediately. Do not surface them as questions or ask for
   permission. Identifying a fault and fixing it are the same task.
7. **New folder protocol:** After creating any folder, immediately run
   `python _meta/scripts/scaffold_manifest.py [path] --update-parent`
   to generate a MANIFEST.md stub. No folder is complete without MANIFEST.md.
   At session end, run `python _meta/scripts/validate_manifests.py` to catch any missed.
