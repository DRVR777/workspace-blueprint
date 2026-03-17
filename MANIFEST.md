# MANIFEST — workspace-blueprint (root)

## Envelope

| Field | Value |
|-------|-------|
| `id` | workspace-blueprint-root |
| `type` | workspace |
| `depth` | 0 |
| `parent` | none — this is the root |
| `version` | 1.0.0 |
| `status` | active |

## What I Am
A multi-project agent-native workspace. Self-building system driven by PRDs.
Contains shared infrastructure (`_meta/`, `_core/`), a project template,
and a self-improvement engine (`programs/workspace-builder/`).
Example Acme DevRel workspaces remain as teaching references.

## What I Contain

| Name | Type | Status | Purpose |
|------|------|--------|---------|
| CLAUDE.md | file | active | Always-loaded map: folder structure, naming, placement rules |
| CONTEXT.md | file | active | Task router: task → workspace |
| START-HERE.md | file | active | Human onboarding guide for this template |
| _meta/ | folder | active | Layer 0: ur-prompt, gap registry, gap detection, runner, prd-intake, spec-review, scripts |
| _registry/ | folder | active | Layer 1: task type → workspace mapping, quality tracking, unknown type protocol |
| _core/ | folder | active | Workspace-wide conventions — single source of truth for all architectural patterns |
| _intake/ | folder | active | Inbound PRD queue — drop PRD here or use `intake:` trigger |
| setup/ | folder | active | Onboarding: two-pass questionnaire generates voice rules and domain summary |
| programs/ | folder | active | All projects — contains _template, workspace-builder, and user projects |
| writing-room/ | folder | active | Example workspace: ideas → polished drafts (Acme DevRel) |
| production/ | folder | active | Example workspace: drafts → deliverables (Acme DevRel) |
| community/ | folder | active | Example workspace: content → newsletters/social/events (Acme DevRel) |
| _examples/ | folder | active | Teaching examples — not part of live workflow |
| claude-office-skills-ref/ | folder | active | Office document skills (docx, pdf, pptx, xlsx) |
| campaigns/ | folder | active | Campaign archives: user requirements, agent interpretations, progress docs for each initiative |
| _archive/ | folder | active | Long-term archive for deprecated content and session history files |
| _bus/ | folder | active | Inter-agent communication: broadcast channel, per-agent inboxes, convention checker |

## What I Need From Parent
Nothing — this is the root. All context is self-contained.

## What I Give To Children
- Naming conventions (CLAUDE.md)
- File placement rules (CLAUDE.md)
- Cross-workspace flow map (CLAUDE.md + CONTEXT.md)
- Task → workspace routing (CONTEXT.md)
- Token management rules (CLAUDE.md)
- Skills and MCPs inventory (CLAUDE.md)
- Domain-agnostic orientation and prompt generation (_meta/ur-prompt.md)

## What I Return To Parent
This workspace is standalone. No parent to return to.

## Routing Rules

| Condition | Go To |
|-----------|-------|
| Writing, editing, or refining text content | writing-room/ |
| Building demos, specs, or build pipeline | production/ |
| Creating newsletters, social, events | community/ |
| Routing a task to the right workspace | `_registry/REGISTRY.md` |
| Generating a prompt for any task | _meta/ |
| Detecting gaps after a session | _meta/gaps/ |
| Learning or teaching the template | _examples/ |
| Creating PPTX, DOCX, XLSX, or PDF files | claude-office-skills-ref/ |
| Look up an architectural pattern | `_core/CONVENTIONS.md` |
| Onboard a new workspace domain | `setup/questionnaire.md` (or type `setup`) |
| Scaffolding a new project from a PRD | Drop PRD in `_intake/queue/`, run `_meta/prd-intake.md` |
| Starting or ending any session | `_meta/runner.md` |
| Orienting with no prior context | Read this MANIFEST, then CLAUDE.md |

## Gap Status

| Gap ID | Type | Description | Severity |
|--------|------|-------------|----------|
| gap-001 | missing_bridge | production → community handoff undocumented on production side | degrading |
| gap-002 | missing_bridge | writing-room voice.md used by community but writing-room doesn't know this | degrading |
| gap-003 | shallow_node | All 4 pipeline stage folders had no local MANIFEST (closed by Step 3) | blocking |
| gap-004 | missing_composition | No Layer 0 meta-prompt layer existed before _meta/ was created (closed by Step 1) | blocking |

See `_meta/gaps/` for full gap registry and gap objects.
