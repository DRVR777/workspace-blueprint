# PRD Source — workspace-builder

This project builds and maintains the workspace itself.
The source requirements come from three PRDs. Key extractions below.
Full roadmap with implementation status: `roadmap.md`

---

## Source 1: MWP v1 — Model Workspace Protocol

**Core premise:** Folder structure IS the product. Markdown files route agents to right context.
**Key patterns:**
- 4-layer routing: CLAUDE.md (map) → CONTEXT.md (router) → stage CONTEXT.md (scope) → content files
- Stage contracts: Inputs / Process / Outputs on every stage CONTEXT.md
- Stage handoffs via output/ folders — next stage reads previous stage's output/
- One-way cross-references — no circular dependencies
- Selective section routing — load specific sections of files, not full files
- Canonical sources — one home per piece of information
- CONTEXT.md = routing only, never content
- Placeholder system: `{{VARIABLE}}` replaced during onboarding
- Workspace builder: meta-workspace that generates new workspaces from domain descriptions
- Trigger keywords: `setup` (onboarding), `status` (show stage outputs)

**Workspace structure from MWP:**
```
workspace/
  CLAUDE.md           ← layer 0: always-loaded map
  CONTEXT.md          ← layer 1: routing table
  setup/              ← onboarding: questionnaire.md
  brand-vault/        ← hydrated during onboarding
  stages/
    01-name/
      CONTEXT.md      ← Inputs/Process/Outputs contract
      output/         ← stage artifact lands here
      references/     ← reference material for this stage
  shared/             ← cross-stage reference files
```

---

## Source 2: MWP v2 — Battle-Tested Patterns (12 changes)

**Change 1 — Spec format:** Specs define WHAT/WHEN, not HOW. No component names, no frame numbers. Beat Map + Visual Philosophy + Key Moments.

**Change 2 — Checkpoints:** Stage contracts get a Checkpoints section. Agent completes a unit, presents options, human steers. Checkpoints between steps, not within them.

**Change 3 — Voice rules as error conditions:** Three sections: Hard Constraints (numbered errors), Sentence Rules (do/don't pairs with verbatim examples), Pacing (rhythm notation).

**Change 4 — Audit section:** Stage contracts get an Audit section before Outputs. Binary pass/fail checklist agent runs before committing output.

**Change 5 — Value framework:** Content must hit 2+ of: NOVEL, USABLE, QUESTION-GENERATING, INTERESTING. Locked in before drafting.

**Change 6 — Builder creative freedom:** Spec = creative contract (WHAT). Design system = quality floor (minimum standard). Builder has freedom HOW within both.

**Change 7 — Docs over outputs:** Reference docs are authoritative. Previous outputs in output/ are artifacts, not templates. Never learn from older outputs.

**Change 8 — Constants as shared files:** Code workspaces define constants/ folder with colors.ts, timing.ts, typography.ts. All builds import from there.

**Change 9 — Design system as code recipes:** Recipes section with copy-paste patterns. Anti-patterns table. Production checklist.

**Change 10 — Token management:** CLAUDE.md gets a "What to Load" table: task → minimal file set → what NOT to load.

**Change 11 — Questionnaire extracts rules:** Ask for concrete example sentences (right + wrong), not adjectives. Two-pass: quick answers → agent generates draft rules → human edits.

**Change 12 — Specific process steps:** Process steps specific enough that two agents produce structurally similar outputs. "Write full script in one pass, then audit against voice hard constraints" not "write the script."

---

## Source 3: ALWS — Adaptive Living Workspace System

**Core premise:** Self-extending, self-improving system built from markdown and folder conventions.
No vector databases, no custom orchestration — folder structure IS the orchestration layer.

**Five layers:**
- Layer 0: Ur-Prompt — domain-agnostic, generates execution prompts from gap objects
- Layer 1: Meta-Registry — routes and scaffolds workspaces, REGISTRY.md
- Layer 2: Workspace Orchestrator — CLAUDE.md (depth-1 only) + root CONTEXT.md
- Layer 3: Stage Pipeline — stage contracts with faceted context loading
- Layer 4: Artifact Surface — output files

**Terry Davis Constraint:** Every layer must work equally for Byzantine tax law, protein folding, and jazz harmony. Domain knowledge must not leak upward.

**Fractal MANIFEST:** Every folder at every depth has MANIFEST.md with identical schema (id, type, depth, parent, status, What I Am, What I Contain, What I Need From Parent, What I Give To Children, What I Return To Parent, Routing Rules).

**Gap system:** Three types (missing_composition, missing_bridge, shallow_node). Inference log (pending.txt) → gap detection agent → formal gap objects → runner executes. Closed loop.

**Runner:** Autonomous. Reads gap registry. Picks highest-severity gap. Feeds to Ur-Prompt. Executes. Closes. Repeats. Stops at `requires_human: true` gaps.

**PRD intake:** Extract (project name, programs, contracts, decisions, unknowns) → collision check → log all inferences → scaffold (in order: root MANIFEST → planning → meta → shared → programs) → report.

**Quality gates:** ADR distinction (accepted vs assumption). Assumption blocks building. Spec review (4 checks: ADR gate, contract gate, coherence, gap gate). Status model (scaffold → specced → active → complete).

**Knowledge graph:** Distributed across file headers. Edges typed (requires, instantiates, enables, contradicts, refines). Materialized to graph.json by graph-builder agent. Enables traversal, gap detection by orphan check, path finding.

**Improvement engine:** Aggregates telemetry from runs → detects patterns → proposes diffs → validates against historical data → auto-applies low-risk changes → human-gates high-impact changes. Diff log per stage tracks changes with evidence.

**Intake pipeline (5 stages):** envelope (wrap with ID/hash) → classify (type detection, confidence score) → route (MANIFEST-driven, hop-by-hop) → place (write file + update all MANIFESTs + update graph) → update (log routing confidence).

**Registry:** REGISTRY.md maps task types to workspaces. New task types trigger workspace-builder pipeline. Avg quality tracked per workspace type.

---

## Implementation Priority

See `roadmap.md` for the complete requirements table with current status.
Priority order (from roadmap):
1. Complete MANIFEST.md at every routing folder
2. _core/CONVENTIONS.md (single source of truth for all patterns)
3. Token management (What to Load) in CLAUDE.md templates
4. Checkpoints + Audit sections in stage templates
5. Onboarding questionnaire system
6. setup/status trigger keywords
7. 5-stage intake pipeline
8. Knowledge graph system
9. Improvement engine
10. Registry system
