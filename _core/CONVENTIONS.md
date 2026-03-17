# CONVENTIONS — Workspace Architectural Patterns

Single source of truth for all patterns in this workspace.
Sources: MWP v1 (9 patterns) + MWP v2 (12 changes) + ALWS (fractal system).
All templates, agents, and CONTEXT.md files reference this document.

**Rule:** When a pattern here conflicts with a local file, this file wins.
Update local files to match — do not update this file to match locals.

---

## Pattern Tiers — Read This Before the Index

28 patterns is a lot. Read by tier. Stop when you have what you need.

| Tier | Name | Patterns | When to read |
|------|------|----------|--------------|
| **Tier 1 — Foundation** | Must know before anything | P-01, P-02, P-05, P-19, P-21 | First session in any workspace |
| **Tier 2 — Enforcement** | Must know before building | P-04, P-10, P-11, P-22, P-23, P-24, P-25 | Before writing any code or spec |
| **Tier 3 — Quality** | Must know before finishing | P-12, P-13, P-14, P-15, P-17, P-27 | During review and audit phases |
| **Tier 4 — Refinement** | Read when you hit the situation | Everything else | On demand only |

---

## Index

| # | Pattern | Source | Section |
|---|---------|--------|---------|
| P-01 | 4-Layer Routing | MWP v1 | §1 |
| P-02 | Stage Contracts | MWP v1 | §2 |
| P-03 | Stage Handoffs via output/ | MWP v1 | §3 |
| P-04 | One-Way Cross-References | MWP v1 | §4 |
| P-05 | Canonical Sources | MWP v1 | §5 |
| P-06 | CONTEXT.md = Routing Only | MWP v1 | §6 |
| P-07 | Placeholder System | MWP v1 | §7 |
| P-08 | Selective Section Routing | MWP v1 | §8 |
| P-09 | Trigger Keywords | MWP v1 | §9 |
| P-10 | Spec as Contract (WHAT/WHEN not HOW) | MWP v2 Change 1 | §10 |
| P-11 | Checkpoints in Stage Contracts | MWP v2 Change 2 | §11 |
| P-12 | Voice Rules as Error Conditions | MWP v2 Change 3 | §12 |
| P-13 | Audit Section in Stage Contracts | MWP v2 Change 4 | §13 |
| P-14 | Value Framework | MWP v2 Change 5 | §14 |
| P-15 | Docs Over Outputs | MWP v2 Change 7 | §15 |
| P-16 | Constants as Shared Files | MWP v2 Change 8 | §16 |
| P-17 | Token Management (What to Load) | MWP v2 Change 10 | §17 |
| P-18 | Specific Process Steps | MWP v2 Change 12 | §18 |
| P-19 | Fractal MANIFEST Protocol | ALWS | §19 |
| P-20 | Terry Davis Constraint | ALWS | §20 |
| P-21 | Gap System (3 types + closed loop) | ALWS | §21 |
| P-22 | Fix-First Rule | ALWS | §22 |
| P-23 | ADR Distinction (accepted vs assumption) | ALWS | §23 |
| P-24 | Status Model | ALWS | §24 |
| P-25 | New Folder Protocol | System | §25 |
| P-26 | 5-Stage Intake Pipeline | ALWS §11 | §26 |
| P-27 | Builder Creative Freedom (Spec + Quality Floor) | MWP v2 Change 6 | §27 |
| P-28 | Design System as Code Recipes | MWP v2 Change 9 | §28 |

---

## §1 — P-01: 4-Layer Routing

```
Layer 0: CLAUDE.md          ← always-loaded map — depth 1 only
Layer 1: CONTEXT.md         ← routing table — sends agent to correct sub-context
Layer 2: stage CONTEXT.md   ← scoped task: Inputs / Process / Outputs
Layer 3: content files      ← actual content agents read/write
```

**Rule:** Each layer routes to the layer directly below it. Layers never skip.
**Rule:** CLAUDE.md lists only the names and one-line purposes of depth-1 folders. No internals.
**Rule:** Each project has its own CLAUDE.md and CONTEXT.md — the root files never describe project internals.

---

## §2 — P-02: Stage Contracts

Every stage CONTEXT.md must contain these five sections in order:

```markdown
## Inputs
[What this stage reads — file paths or folder names]

## Process
[Numbered steps specific enough that two agents produce structurally similar outputs]

## Checkpoints
[Unit boundaries: what the agent presents to human before proceeding]
See P-11 for Checkpoints detail.

## Audit
[Binary pass/fail checklist agent runs before committing output]
See P-13 for Audit detail.

## Outputs
[What this stage produces — file paths or folder names]
```

**Rule:** Process steps must be specific, not generic. "Write full script in one pass, then audit against voice hard constraints" — not "write the script." (P-18)

---

## §3 — P-03: Stage Handoffs via output/

- Each stage writes its artifact to its own `output/` folder.
- The next stage reads the previous stage's `output/` — it does not reach into the previous stage's working files.
- Handoff is complete when the artifact lands in `output/`.

---

## §4 — P-04: One-Way Cross-References

- File A may reference file B. File B may not reference file A.
- No circular dependencies anywhere in the folder tree.
- Cross-project dependencies are the only exception — they are documented as `missing_bridge` gaps in `_meta/gaps/`, not as direct references.

---

## §5 — P-05: Canonical Sources

- Every piece of information has exactly one home.
- All other files that need that information reference the canonical source — they do not copy it.
- When the canonical source changes, nothing else needs updating.
- **MANIFEST.md** is the canonical source for what a folder contains and what it does.
- **_planning/prd-source.md** is the canonical source for project requirements.
- **_core/CONVENTIONS.md** (this file) is the canonical source for architectural patterns.

---

## §6 — P-06: CONTEXT.md = Routing Only

CONTEXT.md files route agents to content. They never contain content themselves.

**Allowed in CONTEXT.md:**
- Tables: "If you want to do X, go to Y"
- Section headings that match task names
- One-line descriptions of what a linked file contains

**Not allowed in CONTEXT.md:**
- Definitions, explanations, examples, rules, or any information an agent would act on
- Code snippets
- Multi-paragraph prose

**Test:** If removing a sentence from CONTEXT.md would cause an agent to lose capability (not just navigation), that sentence belongs in a content file.

---

## §7 — P-07: Placeholder System

Template files use `{{VARIABLE}}` placeholders replaced during project creation.

| Placeholder | Replaced with | Where |
|-------------|---------------|-------|
| `{{PROJECT_NAME}}` | Slugified project name | All template files |
| `{{CREATED}}` | ISO date (YYYY-MM-DD) | MANIFEST.md envelope |

**Rule:** Placeholders are replaced in the clone, never in `programs/_template/` itself.
**Rule:** `programs/_template/` must always contain unresolved placeholders. If it contains a resolved value, it has been modified incorrectly — restore the placeholder.

---

## §8 — P-08: Selective Section Routing

Agents load specific sections of files, not full files.

**In CONTEXT.md:** Route agents to named sections: "Go to `prd-source.md §2`" not "Go to `prd-source.md`."
**In MANIFEST.md:** "What I Contain" table tells agents exactly which file handles which task.
**In CLAUDE.md:** "What to Load" table (P-17) specifies minimum file set per task type.

---

## §9 — P-09: Trigger Keywords

Keywords that activate agents when typed by a human:

| Keyword | Action | Agent |
|---------|--------|-------|
| `intake: "[text]"` | Treat text as PRD, create new project | `_meta/prd-intake.md` |
| `run gaps` | Process pending.txt, close gaps | `_meta/runner.md` |
| `audit` | Check workspace vs PRD requirements | `programs/workspace-builder/programs/auditor/CONTEXT.md` |
| `setup` | Run onboarding questionnaire | `setup/questionnaire.md` (not yet implemented — P-01 §9) |
| `status` | Show all stage output/ contents | stage output scan (not yet implemented) |

**Rule:** Trigger keywords are documented in root `CLAUDE.md` only. Never in project files.

---

## §10 — P-10: Spec as Contract (WHAT/WHEN not HOW)

Specs define observable outcomes, not implementation approach.

**A spec must answer:**
- WHAT: what is produced (observable artifact or behavior)
- WHEN: what condition triggers or completes the work

**A spec must NOT contain:**
- Component names, file names, frame numbers
- Technology choices (unless locked by ADR)
- Step-by-step instructions (those go in Process section of stage contract)

**Format for spec documents:**
```markdown
## Beat Map
[Sequence of observable beats/moments — what happens, not how]

## Key Moments
[The 2-5 moments the output must nail — observable criteria]

## Visual Philosophy (or equivalent domain term)
[Principles the builder must honor — quality floor, not prescriptive]
```

---

## §11 — P-11: Checkpoints in Stage Contracts

Checkpoints = pause points where the agent presents a unit of work and waits for human steering.

**Rules:**
- Checkpoints appear between steps, never within a step.
- Each checkpoint names what the agent presents and what choices the human has.
- Agent does not proceed past a checkpoint without human input.

**Format:**
```markdown
## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 2 | [what is shown] | approve / revise / restart |
| Step 4 | [what is shown] | approve / revise with notes |
```

---

## §12 — P-12: Voice Rules as Error Conditions

Voice and tone rules are enforced as machine-checkable error conditions — not style guides.

**Three sections:**

**Hard Constraints** (numbered, binary):
```
ERROR-01: [prohibited pattern] — [why it breaks the voice]
ERROR-02: ...
```

**Sentence Rules** (do/don't pairs with verbatim examples):
```
DO: "We tested this across 47 production deployments."
DON'T: "This approach has been validated extensively."
WHY: Specific numbers beat vague affirmations.
```

**Pacing** (rhythm notation):
```
Long sentence. Short sentence. Fragment. Then the payoff lands.
Never: three long sentences in a row.
```

**Rule:** Voice rules are extracted from questionnaire answers (P-11 §18), not invented by agents.
**Rule:** Store in `shared/voice.md` at the project level.

---

## §13 — P-13: Audit Section in Stage Contracts

Every stage contract ends with a binary pass/fail Audit checklist the agent runs before writing to `output/`.

**Format:**
```markdown
## Audit
Before committing to output/, verify:
- [ ] [observable check 1]
- [ ] [observable check 2]
- [ ] [observable check 3]

All items must pass. If any fail: fix before committing, log repair to _meta/gaps/pending.txt.
```

**Rule:** Audit items must be binary (pass/fail), not subjective ("is the quality good").
**Rule:** Failure does not block — it triggers a fix cycle, then re-audit.

---

## §14 — P-14: Value Framework

Content must hit 2 or more of these four criteria before drafting begins:

| Label | Definition |
|-------|------------|
| NOVEL | Makes a connection or claim the audience has not seen before |
| USABLE | Gives the audience something they can act on immediately |
| QUESTION-GENERATING | Makes the audience wonder something they'll want to resolve |
| INTERESTING | Compelling to read independent of utility |

**Rule:** Value framework check happens in the spec phase (P-10), before any Process steps.
**Rule:** If a draft doesn't hit 2+, revise the spec — not the draft.

---

## §15 — P-15: Docs Over Outputs

Reference docs are authoritative. Output artifacts are artifacts.

**Rule:** Agents never learn patterns from files in `output/` folders.
**Rule:** When a previous output and a reference doc conflict, the reference doc wins.
**Rule:** `output/` files may be read to continue in-progress work, but never as templates.

---

## §16 — P-16: Constants as Shared Files

In code workspaces, constants live in `shared/constants/`:

| File | Contains |
|------|----------|
| `colors.ts` | All color tokens |
| `timing.ts` | All animation/delay values |
| `typography.ts` | All font/size/weight tokens |

**Rule:** No magic strings or numbers in program source code. Import from shared constants.
**Rule:** When the same value appears in two program files, it belongs in constants.

---

## §17 — P-17: Token Management (What to Load)

Every project CLAUDE.md must contain a "What to Load" table that specifies the minimum file set per task type. Prevents agents from loading unnecessary context.

**Format:**
```markdown
## What to Load

| Task | Load these files | Do NOT load |
|------|-----------------|-------------|
| [task type] | [file 1], [file 2] | [file 3], [file 4] |
```

**Rules:**
- Default to the minimum. Add files only when the task requires them.
- Never load all files "just in case."
- Output artifacts (`output/`) are never loaded unless continuing in-progress work.

**Template:** See `programs/_template/CLAUDE.md` for a starter "What to Load" table.

---

## §18 — P-18: Specific Process Steps

Process steps must be specific enough that two independent agents produce structurally similar outputs.

**Failing example (too generic):**
```
3. Write the script.
```

**Passing example (specific):**
```
3. Write the full script in one pass without stopping.
   Then run Audit checklist (§Audit below).
   If any item fails: fix in place, re-run that checklist item only.
```

**Test:** Give the Process section to an agent with no other context. If the output would vary widely, the steps are not specific enough.

---

## §19 — P-19: Fractal MANIFEST Protocol

Every folder that an agent navigates to has a MANIFEST.md with this schema:

```markdown
# MANIFEST — [path/]

## Envelope
| Field | Value |
|-------|-------|
| `id` | [unique-id] |
| `type` | [root | programs-container | project | planning | meta | programs | program | gaps | conventions] |
| `depth` | [0=root, 1=workspace folders, 2=projects in programs/, 3=project subfolders, 4=programs within projects] |
| `parent` | [parent folder path] |
| `status` | [scaffold | specced | active | complete | template | reference] |

## What I Am
[1-2 sentence description]

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| [file or folder] | [file|folder] | [one-line purpose] |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| [task] | [file or folder] |
```

**What I Need From Parent** and **What I Return To Parent** sections are optional — add when the folder is a stage in a pipeline.

**Leaf folders** (no sub-navigation needed: `adr/`, `queue/`, `processed/`) may omit MANIFEST.md.

**Depth values:**
| Depth | Examples |
|-------|---------|
| 0 | workspace root |
| 1 | `_meta/`, `programs/`, `_core/`, `_intake/` |
| 2 | `programs/workspace-builder/`, `programs/_template/` |
| 3 | `programs/workspace-builder/_planning/`, `programs/workspace-builder/_meta/` |
| 4 | `programs/workspace-builder/programs/auditor/` |

---

## §20 — P-20: Terry Davis Constraint

Every layer of the system must work equally for Byzantine tax law, protein folding, and jazz harmony.

**Test:** Replace all domain-specific nouns in a file with "[DOMAIN NOUN]". If the file still makes sense as an orchestration or routing instruction, it passes. If it breaks down (depends on domain knowledge), it fails.

**Rule:** Domain knowledge must not leak upward past the stage CONTEXT.md it belongs to.
**Rule:** Root-level files (CLAUDE.md, CONTEXT.md, MANIFEST.md) must pass the Terry Davis test.
**Rule:** `_meta/` agents must pass the Terry Davis test — they are domain-agnostic infrastructure.

## Known Exception

Root `CLAUDE.md` is exempt from the Terry Davis test for its **"What I Contain" / Projects table only**, because it must enumerate actual project names to be useful. All other sections of root-level files must still pass the test. This exception does not propagate downward — project-internal files have no exemption.

## Known Mechanical Limitation

The Terry Davis test requires human judgment at two points: (1) deciding which nouns count as "domain-specific," and (2) deciding whether the post-replacement file "still works as orchestration." There is no automated version of this test. Sessions that create new root-level or `_meta/` files must include at least one human-run Terry Davis check before closing.

---

## §21 — P-21: Gap System (3 types + closed loop)

**Three gap types:**

| Type | Definition | Example |
|------|-----------|---------|
| `missing_composition` | Node exists, but internal structure is unknown | Folder exists, no MANIFEST |
| `missing_bridge` | Two nodes exist, but no documented edge between them | Two projects share a contract, no cross-reference |
| `shallow_node` | Node exists but is not deep enough to be actionable | CONTEXT.md has routing rows but no content behind them |

**Closed loop:**
```
1. Run produces inference → log to [scope]/_meta/gaps/pending.txt
2. gap-detection-agent.md classifies pending entries → writes gap JSON objects
3. runner.md reads gap registry → picks highest-severity open gap
4. runner feeds gap to ur-prompt.md → execution prompt generated
5. Agent executes → marks gap closed
6. Repeat until no open gaps with requires_human: false
```

**Scope rules:**
- Project-internal inference → `programs/[project]/_meta/gaps/pending.txt`
- Cross-project inference → `{root}/_meta/gaps/pending.txt`

**Severity tiers:**
| Tier | Meaning | Runner behavior |
|------|---------|-----------------|
| critical | Blocks all progress | Fix before any other gap |
| high | Blocks a pipeline stage | Fix before moving to next stage |
| medium | Reduces quality | Fix in current session if possible |
| low | Cosmetic or optional | Queue for later |

---

## §22 — P-22: Fix-First Rule

When an agent identifies an error, broken reference, stale content, or missing file:

1. Fix it immediately.
2. Log the repair to the nearest `_meta/gaps/pending.txt`.
3. Do not surface the error as a question.
4. Do not ask permission.

**Identifying a fault and fixing it are the same task.**

The only exception: fixes that require human judgment (e.g., resolving an `assumption`-status ADR). These are logged with `requires_human: true`.

## Scope Boundary

Fix-First applies within the SCOPE of the current session task.

- Error discovered **inside** current task's scope → fix immediately, then continue.
- Error discovered **outside** current task's scope → log to nearest `_meta/gaps/pending.txt` with severity `medium`, then continue the current task.

The session runner declares scope at session start. Errors outside that scope are not ignored — they are deferred, not skipped. They will be picked up in a dedicated maintenance session.

---

## §23 — P-23: ADR Distinction (accepted vs assumption)

Architecture Decision Records have two valid statuses:

| Status | Source | Effect |
|--------|--------|--------|
| `accepted` | Explicitly stated in PRD | Building may proceed |
| `assumption` | Inferred by agent | **Blocks all building until human validates** |

**Rule:** If a decision was not explicit in the PRD, it is an `assumption`. Do not write it as `accepted`.
**Rule:** `assumption`-status ADRs are surfaced to the human immediately and logged as `requires_human: true` gaps.
**Rule:** Building past an unvalidated `assumption` produces a gap with severity: `critical`.

## Blocking vs Non-Blocking Threshold

Not all assumptions block building. Use this test:

**Blocking assumption** (must validate before building): The decision affects a system BOUNDARY — network protocol shape, database schema, shared contract interface, public API. Changing it later requires touching files in more than one program.

**Non-blocking assumption** (log and continue): The decision is internal to a single program. Changing it later requires only code changes within that program with no effect on any contract or interface.

When uncertain: if changing the decision would require updating any file in `shared/` or `_meta/contracts/`, it is blocking. Otherwise it is non-blocking.

**ADR template:**
```markdown
# ADR-[number]: [title]

Status: assumption | accepted
Date: YYYY-MM-DD
Source: [PRD section or "inferred"]

## Decision
[What was decided]

## Rationale
[Why — from PRD text if accepted, from inference logic if assumption]

## Consequences
[What this enables and what it forecloses]
```

---

## §24 — P-24: Status Model

**Projects** (in `programs/`):
| Status | Meaning | Gate to next |
|--------|---------|-------------|
| `scaffold` | Cloned from template, placeholders replaced | spec-review.md passes all 4 checks |
| `specced` | PRD requirements formalized, ADRs accepted | Human confirms readiness |
| `active` | Work in progress | All programs reach `complete` |
| `complete` | All outputs produced, no open critical gaps | — |

**Programs** (stages within a project):
| Status | Meaning |
|--------|---------|
| `scaffold` | CONTEXT.md and MANIFEST.md exist |
| `specced` | Stage contract fully written |
| `active` | Currently being executed |
| `complete` | output/ populated, Audit section passed |

**Template:**
| Status | Meaning |
|--------|---------|
| `template` | Source for cloning — never populate directly |

**Rule:** `programs/_template/` status is always `template`. If it shows any other status, the file has been modified incorrectly.

---

## §25 — P-25: New Folder Protocol

**Rule:** Every folder an agent creates must have a MANIFEST.md before the session ends.

**Process when creating any folder:**
1. Create the folder.
2. Immediately run: `python _meta/scripts/scaffold_manifest.py [folder-path] --update-parent`
3. Open the generated MANIFEST and fill in: "What I Am" description, purpose column in "What I Contain", Routing Rules rows.

**If the script is unavailable:** Create MANIFEST.md manually using the schema from P-19 (§19).

**Session-end enforcement:** `runner.md` ending protocol Step 2 runs `validate_manifests.py` to catch any missed folder stubs. All ERRORs (missing MANIFESTs) must be resolved before the session closes.

**Why:** Agents navigate by reading MANIFEST Routing Rules. A folder without MANIFEST.md is invisible — the routing chain breaks and the next agent gets lost.

---

## §26 — P-26: 5-Stage Intake Pipeline

All documents entering the workspace flow through five stages:

| Stage | Name | What happens |
|-------|------|-------------|
| 01 | Envelope | Assign doc-id, capture source metadata, write envelope JSON |
| 02 | Classify | Detect document type (prd, feature-request, adr, contract-update, question, unknown), assign confidence score |
| 03 | Route | MANIFEST-driven hop-by-hop routing to correct handler and target folder |
| 04 | Place | Execute handler, create files, update all MANIFESTs |
| 05 | Update | Archive envelope, log intake event, log routing confidence |

**Entry point:** `_meta/intake-pipeline/stage-01-envelope/CONTEXT.md`
**Shortcut for PRDs:** `_meta/prd-intake.md` (executes Stage 04 directly, skipping 01-03)

**Rules:**
- Stages execute in order. Stage N always reads from Stage N-1's output.
- `question` and `unknown` type documents stop at Stage 03 and log to pending.txt.
- Low-confidence classifications (< 0.85) proceed but log a review flag to pending.txt.
- Every document that enters Stage 01 must exit Stage 05 (or be logged as `stopped`).

**Confidence threshold:** 0.7 minimum to proceed. Below 0.7 → classified as `unknown`.

---

## §27 — P-27: Builder Creative Freedom (Spec + Quality Floor)

The spec and the design system are two separate constraints. The builder has full creative freedom in the space between them.

```
Spec     = WHAT must be achieved (observable outcomes) — ceiling constraint
Design   = quality floor (minimum acceptable standard) — floor constraint
Builder  = free to decide HOW within the space between floor and ceiling
```

**Rule:** A spec that specifies HOW is a broken spec. Convert implementation details to observable outcomes.
**Rule:** A design system that specifies outcomes is a broken design system. Convert outcomes to quality standards.
**Rule:** When reviewing builder output, check only: (1) does it achieve the spec? (2) does it meet the quality floor? If yes to both, the HOW is not subject to review.

**Anti-pattern:** "Use a 3-column grid" in a spec. That's a HOW.
**Correct:** "Information hierarchy is immediately clear to a first-time reader." That's a WHAT.

**Anti-pattern:** "Mention the product name in the first paragraph" in a design system. That's a WHAT.
**Correct:** "Brand voice is present in the first 50 words." That's a quality floor.

---

## §28 — P-28: Design System as Code Recipes

Design systems in code workspaces are expressed as copy-paste recipes with anti-patterns, not as prose guidelines.

**Structure:**
```markdown
## Recipes
### [recipe-name]
\`\`\`typescript
// copy-paste this exact pattern
\`\`\`

## Anti-Patterns
| Do NOT do this | Do this instead | Why |
|----------------|-----------------|-----|
| [bad pattern] | [good pattern] | [reason] |

## Production Checklist
- [ ] [binary check 1]
- [ ] [binary check 2]
```

**Rule:** Every recipe must be copy-paste ready — no placeholders that require interpretation.
**Rule:** Anti-patterns table must show the actual wrong code or pattern, not a description of it.
**Rule:** A design system without a Production Checklist is incomplete (agents have no gate to check against).
**Rule:** Store in `shared/design-system.md` at the project level.
**Cross-reference:** Constants referenced in recipes must live in `shared/constants/` (P-16 §16).
