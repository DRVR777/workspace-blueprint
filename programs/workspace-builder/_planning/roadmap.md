# Workspace Build Roadmap — PRD Requirements vs Implementation

Source PRDs: MWP v1, MWP v2 (battle-tested patterns), ALWS
Last audited: 2026-03-14
Overall status: ALL 10 BUILD PRIORITIES COMPLETE. Build loop is idle until new PRDs arrive.
Two ⚠️ items are accepted gaps (see notes) — not open work.

Legend: ✅ implemented | ⚠️ accepted gap (not open work) | ❌ not implemented

---

## ALWS Core Architecture

| Requirement | Source | Status | Location / Notes |
|-------------|--------|--------|-----------------|
| Ur-Prompt (Layer 0 generator) | ALWS §4 | ✅ | `_meta/ur-prompt.md` |
| Fractal MANIFEST at every folder | ALWS §6 | ⚠️ accepted | Key folders covered; P-25 (new folder protocol) handles new folders on creation. Deep sub-folders done on-demand. Not a blocker. |
| Root CLAUDE.md depth-1 only | ALWS §7 | ✅ | `CLAUDE.md` |
| Root CONTEXT.md routing table | ALWS §7 | ✅ | `CONTEXT.md` |
| Gap detection engine | ALWS §10 | ✅ | `_meta/gap-detection-agent.md` |
| Gap schema (3 types) | ALWS §10 | ✅ | `_meta/gap-schema.json` |
| Gap registry with severity tiers | ALWS §10 | ✅ | `_meta/gaps/CONTEXT.md` |
| Inference log (pending.txt) | ALWS §10 | ✅ | `_meta/gaps/pending.txt` |
| Runner (autonomous gap resolution) | ALWS §13 | ✅ | `_meta/runner.md` |
| PRD intake pipeline | ALWS §12 | ✅ | `_meta/prd-intake.md` |
| Status transitions (scaffold→specced→active→complete) | ALWS §12 | ✅ | `_meta/status-transitions.md` |
| Spec review gate (4 checks) | ALWS §12 | ✅ | `_meta/spec-review.md` |
| ADR distinction (accepted vs assumption) | ALWS §12 | ✅ | All `_planning/CONTEXT.md` files |
| `programs/` folder structure | ALWS §3 | ✅ | `programs/` |
| Template cloning system | ALWS §12 | ✅ | `programs/_template/`, `_meta/scripts/new_project.py` |
| `intake:` trigger keyword | ALWS §11 | ✅ | `CLAUDE.md` |
| Python automation scripts | ALWS §3 | ✅ | `_meta/scripts/` — new_project.py, run_gaps.py, scaffold_manifest.py, status.py, validate_manifests.py |
| Knowledge graph system | ALWS §9 | ✅ | `_meta/graph-engine/` — workspace-level doc relationship schema (CONTEXT.md, edge-schema.json, graph-schema.json). NOTE: distinct from `programs/knowledge-graph/` which is the full CDS project (4 built programs, MCP server) — that is a child project, not workspace-builder infrastructure. |
| 5-stage intake pipeline (envelope/classify/route/place/update) | ALWS §11 | ✅ | `_meta/intake-pipeline/` — 5 stage CONTEXT.md files with full contracts |
| Improvement engine | ALWS §14 | ✅ | `_meta/improvement-engine/` — CONTEXT.md, telemetry-schema.json, diff-log-schema.json |
| Type detection and registry | ALWS §5 | ✅ | `_registry/` — REGISTRY.md, CONTEXT.md, quality-schema.json |
| Telemetry system | ALWS §14 | ✅ | Schema at `_meta/improvement-engine/telemetry-schema.json` — stages emit to `output/telemetry.json` |
| finalized.flag cascade protection | ALWS §15 | ✅ | `_meta/guards/guard-03-finalized-flag.md` — cascade check protocol, audit, human-only flag creation |
| Reconciliation agent (MANIFEST staleness) | ALWS §6 | ✅ | `_meta/guards/guard-05-manifest-reconcile.md` — auto-fix additions, log ghost/status gaps, integrated into runner.md |
| `{root}/_registry/` workspace registry | ALWS §5 | ✅ | `_registry/` — task types registered, quality tracking, unknown type protocol |

---

## MWP v1 Core Architecture

| Requirement | Source | Status | Location / Notes |
|-------------|--------|--------|-----------------|
| 4-layer routing (CLAUDE→CONTEXT→stage CONTEXT→content) | MWP v1 | ✅ | Implemented |
| Stage contracts (Inputs/Process/Outputs) | MWP v1 | ✅ | In prd-intake.md templates |
| Stage handoffs via output/ folders | MWP v1 | ✅ | In stage MANIFEST templates |
| One-way cross-references | MWP v1 | ✅ | Enforced by convention |
| Selective section routing in CONTEXT.md | MWP v1 | ✅ | All CONTEXT.md files |
| Canonical sources (MANIFEST as single source of truth) | MWP v1 | ✅ | MANIFEST.md at every key folder |
| CONTEXT.md = routing only, not content | MWP v1 | ✅ | Enforced across all CONTEXT.md files |
| Placeholder system `{{VARIABLE}}` | MWP v1 | ✅ | `programs/_template/` uses placeholders |
| Onboarding questionnaire system | MWP v1 | ✅ | `setup/questionnaire.md` — two-pass: quick answers → agent drafts rules → human edits |
| `setup` trigger keyword for onboarding | MWP v1 | ✅ | In root `CLAUDE.md` Trigger Keywords |
| `status` trigger keyword | MWP v1 | ✅ | In root `CLAUDE.md` Trigger Keywords — scans output/ folders |
| script-to-animation workspace (reference implementation) | MWP v1 | ❌ | Out of scope for this system |
| workspace-builder as a workspace | MWP v1 | ✅ | `programs/workspace-builder/` |
| _core/CONVENTIONS.md (patterns reference) | MWP v1 | ✅ | `_core/CONVENTIONS.md` — 24 patterns across MWP v1, v2, ALWS |
| _core/templates/ folder | MWP v1 | ⚠️ accepted | `programs/_template/` fully serves this role. A separate `_core/templates/` would be redundant. Accepted gap. |

---

## MWP v2 Battle-Tested Patterns

| Requirement | Source | Status | Location / Notes |
|-------------|--------|--------|-----------------|
| Spec format as contracts not blueprints (WHAT/WHEN not HOW) | MWP v2 Change 1 | ✅ | `_core/CONVENTIONS.md` §10 (P-10) |
| Checkpoints section in stage contracts | MWP v2 Change 2 | ✅ | Added to program CONTEXT.md template in prd-intake.md + _core/CONVENTIONS.md §11 |
| Voice rules as error conditions (hard constraints + do/don't pairs) | MWP v2 Change 3 | ✅ | CONVENTIONS.md §12 + `programs/_template/shared/voice.md` scaffold |
| Audit section in stage contracts | MWP v2 Change 4 | ✅ | Added to program CONTEXT.md template in prd-intake.md + _core/CONVENTIONS.md §13 |
| Value framework (NOVEL/USABLE/QUESTION-GENERATING/INTERESTING) | MWP v2 Change 5 | ✅ | CONVENTIONS.md §14 + added to Audit checklist in prd-intake.md program CONTEXT template |
| Builder creative freedom within quality floor | MWP v2 Change 6 | ✅ | `_core/CONVENTIONS.md` §27 (P-27) |
| "Docs over outputs" rule (don't learn from previous outputs) | MWP v2 Change 7 | ✅ | `_core/CONVENTIONS.md` §15 (P-15) |
| Constants as shared files pattern | MWP v2 Change 8 | ✅ | `_core/CONVENTIONS.md` §16 (P-16) |
| Design system as code recipes | MWP v2 Change 9 | ✅ | `_core/CONVENTIONS.md` §28 (P-28) |
| Token management guidance in CLAUDE.md (What to Load table) | MWP v2 Change 10 | ✅ | In `programs/_template/CLAUDE.md` + CLAUDE.md template in prd-intake.md + _core/CONVENTIONS.md §17 |
| Questionnaire extracts rules not descriptions | MWP v2 Change 11 | ✅ | `setup/questionnaire.md` two-pass system extracts concrete rules |
| Specific process steps (not generic verbs) | MWP v2 Change 12 | ✅ | `_core/CONVENTIONS.md` §18 (P-18) |

---

## Build Priority Order

Based on blocking dependencies and impact:

| Priority | Requirement | Status | Why |
|----------|-------------|--------|-----|
| 1 | Complete MANIFEST.md at every routing folder | ✅ done | Fractal protocol — agents can get lost |
| 2 | _core/CONVENTIONS.md | ✅ done | Single source of truth for all patterns |
| 3 | Token management (What to Load) in CLAUDE.md templates | ✅ done | Context discipline for every agent |
| 4 | Checkpoints + Audit sections in stage templates | ✅ done | Quality gate on every generated program |
| 5 | Onboarding questionnaire system | ✅ done | setup/questionnaire.md two-pass system |
| 6 | `setup` and `status` trigger keywords | ✅ done | UX completeness |
| 7 | 5-stage intake pipeline | ✅ done | `_meta/intake-pipeline/` — 5 stage contracts |
| 8 | Knowledge graph system | ✅ done | `programs/knowledge-graph/` — full CDS spec, 4 programs, 8 ADRs, 3 contracts |
| 9 | Improvement engine | ✅ done | `_meta/improvement-engine/` — telemetry → pattern detection → diff proposals |
| 10 | Registry system | ✅ done | Multi-workspace type detection |

---

## Next Audit
After each build session, re-run `programs/auditor/CONTEXT.md`.
Update Status column above. Add new rows if new PRD requirements are discovered.
