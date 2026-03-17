# Campaign: self-review
## Clarification Document — What Was Meant, What Must Be Done

*This document is the living interpretation of the original prompt. It translates raw intent into structured analysis, maps every bad finding to an action inside the folder architecture, and tracks what gets done.*

---

## ADMIN BYPASS DECLARATION

> **BYPASS ACTIVE**: This campaign has autonomous authority to create files, folders, gap objects, and ADRs within the workspace-blueprint root and any project folder it references. No permission is needed to act on the actions listed below. The only constraint: act within the folder architecture already described in CONVENTIONS.md — do not invent new conventions.

---

## 1. What The Prompt Actually Means

The user asked for a deep critique of the workspace-blueprint — both the system itself and my previous exploratory analysis of it. They want:

1. **25 things that are genuinely strong** — not flattery, not summary. Real reasons the design decisions hold up.
2. **25 things that are genuinely broken, missing, contradictory, or risky** — honest problems, not nitpicks without substance.
3. **For every bad thing: a specific action using the existing folder architecture** — not "you should fix this" but "open this file, write this, here is the exact protocol to follow."
4. **A campaign document** capturing all of this so future sessions can execute the fixes autonomously.

This campaign IS the artifact that addresses point 4. Points 1–3 live below.

---

## 2. The 25 Good Things — With Real Reasons

### GOOD-01: Fractal MANIFEST Protocol (P-19)
Every folder at every depth carries the same self-description schema. The routing information travels WITH the data, not separately. This is exactly how TCP/IP works — each packet knows its own routing information. An agent dropped blindly into `programs/game_engine/world/programs/node-manager/` can reconstruct its position without reading any parent folder. That is genuinely rare in AI system design.

**Why it holds:** Because agents have bounded context windows. Anything that requires reading parent context to understand child context is a design failure in a bounded-context system. MANIFEST solves this.

### GOOD-02: The Terry Davis Constraint (P-20)
"Would this work for Byzantine tax law AND jazz harmony AND protein folding without modification?" is a real test. Most abstraction systems say "we're domain-agnostic" and then have `blog-post` in the variable name on line 4. The Terry Davis Constraint gives you a tool to CATCH that the moment it happens: replace all domain nouns with [DOMAIN NOUN] and see if the file still works. If it doesn't, domain leaked upward.

**Why it holds:** Because it converts a vague principle ("be abstract") into a mechanical test ("run this replace-and-read procedure"). Vague principles are ignored. Tests are checkable.

### GOOD-03: 3-File Navigation System (MANIFEST / CLAUDE / CONTEXT)
Each of the three files has exactly ONE job. MANIFEST = envelope (what I am). CLAUDE = depth-1 map (what lives directly inside me). CONTEXT = router (your task goes HERE). No file does two jobs. No information appears in two files. This is separation of concerns applied to documentation, which almost nobody does.

**Why it holds:** When you split navigation across three purpose-built files, the cognitive load of entering any folder is bounded. You read only what you need: routing? CONTEXT. orientation? MANIFEST. inventory? CLAUDE.

### GOOD-04: CONVENTIONS.md Wins Over All Locals (P-05)
When a pattern in CONVENTIONS.md conflicts with a local file, CONVENTIONS.md wins. The local file is wrong and must be updated. This creates a single enforcement hierarchy — 28 patterns, one source, no exceptions. Most systems degrade because "the local version has evolved" creates divergence that nobody catches until it breaks something in production.

**Why it holds:** Canonical source enforcement is the only reliable way to maintain consistency in a distributed document system. The alternative — "all files are equally authoritative" — produces chaos at scale.

### GOOD-05: Gap System With Exactly 3 Types (P-21)
Three types: missing_composition, missing_bridge, shallow_node. Not 12. Not "whatever makes sense at the time." Three. This is deliberate constraint that prevents the gap system from becoming an infinite-category issue tracker nobody uses because classification takes too long.

**Why it holds:** The three types cover every structural failure mode of a directed graph: a node has unknown internals (missing_composition), two nodes have no edge (missing_bridge), a node exists but isn't deep enough to be useful (shallow_node). The type system is complete because it maps directly to graph theory.

### GOOD-06: Closed-Loop Gap Detection (P-21)
`pending.txt → gap classification → formal gap objects → ur-prompt → execution → gap marked closed`. This is a real feedback loop. Not aspirational. Not "we plan to add this." The files exist: `_meta/gaps/pending.txt`, `_meta/gap-detection-agent.md`, `_meta/ur-prompt.md`, `_meta/runner.md`. Every component is present.

**Why it holds:** Most "improvement systems" stop at the logging step. This one continues through classification, prioritization, prompt generation, and closure tracking. The loop can run without human intervention for any gap with `requires_human: false`.

### GOOD-07: leftOffHere.md — Session Continuity As Engineering Concern
Every session ends by overwriting `leftOffHere.md` with: the exact last file touched per project, the exact next step per project, open questions needing human answers, locked architectural decisions. Start next session: read ONE file. Done.

**Why it holds:** Session state loss is the most common failure in AI-assisted development. Developers lose hours re-orienting. This file treats session continuity as a first-class engineering requirement rather than hoping the agent remembers context.

### GOOD-08: Status Lives ONLY in MANIFEST.md (P-24)
The `status` field exists in exactly one place per project: `MANIFEST.md`. No spreadsheet. No Notion board. No second file tracking status. If MANIFEST says `scaffold`, the project is scaffold. Full stop.

**Why it holds:** Every secondary status tracker eventually diverges from reality. By making MANIFEST the exclusive status source and giving it a machine-readable field, status queries become file reads. Auditing status across all projects becomes a script: read every `programs/*/MANIFEST.md` and extract the `status` field.

### GOOD-09: Quality Gate — Four Binary Conditions (P-24)
scaffold → specced requires: (1) all ADRs accepted or resolved, (2) all contracts defined not stubs, (3) coherence check passed (design doesn't self-contradict), (4) all prerequisites met. Four conditions. All binary. Nothing subjective.

**Why it holds:** Subjective quality gates ("is this good enough?") produce endless arguments and gate-skipping. Four binary conditions produce a checklist. Either all boxes are checked or they're not.

### GOOD-10: ADR Distinction — Accepted vs Assumption (P-23)
Any decision not explicitly stated in the PRD is an `assumption`. Assumption-status ADRs block building. This is the discipline that kills tech debt BEFORE it's created. You can't proceed with "I assumed the database would handle this" — the system forces the assumption into the open.

**Why it holds:** Most projects fail not because of wrong decisions but because of undeclared assumptions that everyone thought were obvious. Making implicit assumptions explicit and blocking on them is the only known way to prevent this failure mode.

### GOOD-11: One-Way Cross-References (P-04)
File A may reference File B. File B may not reference File A. No circular dependencies, anywhere, in the entire folder tree. Cross-project dependencies are documented as `missing_bridge` gaps, not as direct references.

**Why it holds:** Circular references create situations where you cannot load or understand A without loading B and you cannot load or understand B without loading A. In a bounded-context system, this is a fatal deadlock. One-way references guarantee a topological sort always exists.

### GOOD-12: Fix-First Rule (P-22)
When an agent identifies an error, broken reference, stale content, or missing file: fix it immediately. Log the repair. Do not surface it as a question. Do not ask permission. "Identifying a fault and fixing it are the same task."

**Why it holds:** The alternative — "here is a list of errors I found, please tell me what to fix" — creates a second-order problem list that compounds with every session. Fix-First keeps the system from accumulating repair debt.

### GOOD-13: ur-prompt.md as Layer 0
Layer 0 knows NOTHING about what this workspace does. It knows the shape of HOW workspaces are built. It generates prompts for any gap in any workspace of its type. It has a self-test: "Does this file mention anything domain-specific? If yes, it has violated Layer 0."

**Why it holds:** The hardest thing in systems design is keeping abstractions clean. Layer 0 has a self-enforcing constraint — the file itself tells you exactly when it's been violated. That's rare.

### GOOD-14: 5-Stage Intake Pipeline (P-26)
Every document entering the workspace flows through stages 01–05: Envelope → Classify → Route → Place → Update. Every document that enters Stage 01 must EXIT Stage 05 or be logged as `stopped`. Nothing enters and disappears.

**Why it holds:** Information loss is invisible until it matters. The "must exit or be logged stopped" requirement creates accountability for every document. You can always audit: what entered Stage 01 and didn't reach Stage 05?

### GOOD-15: Campaign Folder Captures Full Audit Trail
`campaigns/[project]/prompt.md` = verbatim user requirement. `clarification.md` = agent interpretation. `progress/perfect_final_output.md` = north star for the ideal system. Three documents capture the complete chain from raw intent → interpretation → ideal.

**Why it holds:** Most projects have the requirements in the developer's head or buried in a chat thread. Campaigns externalize that chain of reasoning in a way that any future agent can reconstruct the WHY behind every decision.

### GOOD-16: 5D Semantic Positioning (Knowledge-Graph)
Objects positioned in five dimensions: specificity (abstract→concrete), technicality (intuitive→expert), temporality (foundational→ephemeral), centrality (isolated→hub), confidence (speculative→verified). Navigation is active tool use, not passive retrieval.

**Why it holds:** Standard RAG retrieves by similarity. 5D positioning retrieves by TYPE OF RELATIONSHIP. "Show me everything highly specific but low confidence" is a different query than "show me what is similar to X." The five dimensions create a navigation space that enables queries standard vector search cannot perform.

### GOOD-17: Placeholder System With Strict Enforcement (P-07)
`programs/_template/` always contains unresolved `{{VARIABLE}}` placeholders. If a template file shows a resolved value, it was incorrectly modified. The template is sacred.

**Why it holds:** Template drift is the most common way project templates become useless. When someone "just fixes this one thing" in the template, they've encoded a project-specific decision into the universal source. The "unresolved placeholder = sacred template" rule makes template corruption immediately visible.

### GOOD-18: spec-review.md as the Only Gate
There is exactly ONE path from scaffold to specced: spec-review.md must pass all four checks. Not "the team agreed it was ready." Not "we've been at it long enough." One gate. One path.

**Why it holds:** Multiple gates create negotiation about which gate applies. One gate eliminates negotiation. You can't argue about whether spec-review.md passed or failed — it's a checklist.

### GOOD-19: Stage Handoffs Via output/ Folders (P-03)
Each stage writes to its own `output/` folder. The next stage reads only that `output/` — it never reaches into the previous stage's working files. Handoff is complete when the artifact is in `output/`.

**Why it holds:** Without this rule, stages bleed into each other. Stage N reads Stage N-1's draft and doesn't know it's a draft. The `output/` boundary is a formal interface between stages, equivalent to an API contract between services.

### GOOD-20: Voice Rules as Machine-Checkable Error Conditions (P-12)
`ERROR-01: [prohibited pattern] — [why]`. Binary. Either the pattern appears or it doesn't. Not "the voice should feel warm." Not "avoid being too technical." Named, numbered error conditions.

**Why it holds:** Vague style guides produce vague compliance. Named error conditions produce binary audit results. Agents can check ERROR-01 through ERROR-N mechanically. Humans can check the same list. Both produce the same result.

### GOOD-21: Token Management — "What to Load" Table (P-17)
Every project CLAUDE.md has a "What to Load" table: task → minimum file set → what NOT to load. Loading unnecessary context wastes tokens and dilutes agent focus. This table specifies the minimum.

**Why it holds:** In a workspace with 3,803 files, agents that load "everything that might be relevant" will fail. The "What to Load" table is explicit context window engineering. The agent isn't guessing; it's following a specification.

### GOOD-22: The Workspace Is Self-Hosting
`workspace-builder` is a complete program that manages and improves the workspace. It lives INSIDE the workspace it manages. The system uses itself. This is the strongest possible signal that the architecture is real — it can run on itself.

**Why it holds:** Systems that cannot run on themselves usually have hidden assumptions that make them work "for other domains but not this one." Self-hosting is a correctness proof of the abstraction.

### GOOD-23: Maximum 9 Folders Rule
"Maximum 9 folders in the root of any program. Start with 2. Expand deliberately." This is an explicit structural constraint on cognitive load. If you need 10, you've made an architectural mistake somewhere that needs resolving, not a new folder.

**Why it holds:** Folder proliferation is how systems become unmaintainable. An enforced maximum forces architectural thinking at the point of creation, not after the mess has accumulated.

### GOOD-24: programs/_template Is Sacred
`programs/_template/` is the single source for all new projects. Never populated directly. Always cloned. If `_template/` has a resolved value, it was modified incorrectly and must be restored.

**Why it holds:** A template with one project's concrete values in it is no longer a template — it's a mistake waiting to propagate. The "always clone, never populate directly" rule creates a hard enforcement boundary.

### GOOD-25: P-25 New Folder Protocol — MANIFEST Before Session Ends
"Every folder an agent creates must have a MANIFEST.md before the session ends." The session-end runner validates MANIFESTs. Errors must be resolved before closing.

**Why it holds:** A folder without MANIFEST.md is invisible to the routing system. The routing chain breaks. Future agents get lost. P-25 enforces the invariant that the routing graph is always complete and navigable.

---

## 3. The 25 Bad Things — With Real Reasons + Folder Architecture Actions

### BAD-01: Infrastructure-to-Product Ratio Is Dangerously Skewed
ORACLE is fully specced. NEXUS is fully specced. knowledge-graph has 3 of 4 programs built. But ORACLE has ZERO production code. NEXUS has ZERO production code. The workspace has a mature meta-system managing multiple projects that don't yet produce real outputs. Every session that improves the meta-system INSTEAD of building the products widens this gap.

**What to do:**
- Open `leftOffHere.md`
- Set the top section header to: "BUILD SESSIONS ONLY — meta-system work blocked until oracle Phase 1 complete"
- Add a gap to `_meta/gaps/pending.txt`: `[2026-03-16] | leftOffHere.md | self-review | gap: infrastructure-output ratio inverted — logging BUILD-FIRST constraint`
- Create a formal gap at `_meta/gaps/build-first-constraint.md` with severity: `critical`, type: `shallow_node`, description: "No production output exists from any project despite mature meta-system. Build Phase 1 oracle signal-ingestion before any further meta improvements."

### BAD-02: Terry Davis Constraint Has a Root-Level Paradox
CLAUDE.md at the workspace root lists: "oracle", "game_engine", "knowledge-graph", "workspace-builder". These are domain-specific project names. The Terry Davis Constraint says root-level files must pass the [DOMAIN NOUN] replace test. CLAUDE.md fails this test — it IS domain-specific because it must list the actual projects that exist.

**What to do:**
- Open `_core/CONVENTIONS.md`
- Add a note to §20 (P-20) under a new header `## Known Exception`: "Root CLAUDE.md is exempt from the Terry Davis test for the 'What I Contain' section only, because it must enumerate actual project names. All other sections of root-level files must still pass."
- Document this as a resolved ADR assumption: create `_meta/gaps/terry-davis-root-exception.md` with status: `resolved`, decision: `root CLAUDE.md may list domain-specific project names in its contents table without violating P-20`.

### BAD-03: leftOffHere.md Has a Single Point of Failure — Abrupt Session End
leftOffHere.md says "overwritten at end of every session via wrap up." But `wrap up` is a manually invoked human command. If a session ends abruptly (context limit, API error, network drop, user closes terminal), the state is NOT captured. The file shows the PREVIOUS session's state but appears current because it's always presented as current.

**What to do:**
- Open `_meta/runner.md`
- Add a new section: `## Mid-Session Checkpoint Protocol`: agents must write a PARTIAL leftOffHere.md snapshot every 10 major operations. The partial snapshot header says `STATUS: partial — last checkpoint [timestamp]`. A completed session overwrites with `STATUS: complete`.
- Add a gap: `_meta/gaps/pending.txt`: "leftOffHere.md has no partial-checkpoint mechanism — abrupt session end loses all state since last wrap-up. Severity: high."

### BAD-04: Gap Severity Classification Has No Cross-Validation
When a gap is logged to `pending.txt`, the `gap-detection-agent.md` classifies it as critical/high/medium/low. But the agent that classifies is the same agent (or a similar one) that created the gap. There is no external validation that the severity assignment is correct. A misclassified critical gap could wait in the low-priority queue indefinitely.

**What to do:**
- Open `_meta/gap-detection-agent.md`
- Add a `## Severity Validation Protocol`: "After assigning severity, apply this tiebreaker: if the gap blocks execution of any session task in the current `leftOffHere.md`, minimum severity is `high`. If it blocks spec-review.md from passing for any project, severity is `critical`." This converts severity from a judgment call to a structural derivation.

### BAD-05: 28 Patterns in CONVENTIONS.md — No Progressive Disclosure
CONVENTIONS.md has 28 patterns (P-01 through P-28) all presented at the same weight. An agent (or human) reading CONVENTIONS.md for the first time faces 28 abstractions with no indication of which five are load-bearing vs which are refinements. This is cognitive overload that makes the document harder to internalize and therefore more likely to be partially followed or skipped.

**What to do:**
- Open `_core/CONVENTIONS.md`
- Add a `## Pattern Tiers` section at the top, before the index:
  - **Tier 1 — Foundation (must know before anything):** P-01, P-02, P-05, P-19, P-21
  - **Tier 2 — Enforcement (must know before building):** P-04, P-10, P-11, P-22, P-23, P-24, P-25
  - **Tier 3 — Quality (must know before finishing):** P-12, P-13, P-14, P-15, P-17, P-27
  - **Tier 4 — Refinement (read when you hit the situation):** everything else
- Update the Index table to add a `Tier` column.

### BAD-06: MANIFEST.md Goes Stale Immediately After Creation
P-25 says create MANIFEST.md when you create the folder. But MANIFEST describes what the folder CONTAINS. If you create the MANIFEST when the folder is empty and then add files, the MANIFEST is immediately wrong. The spec says "run validate_manifests.py at session end" — but this only catches MISSING MANIFESTs, not STALE ones.

**What to do:**
- Open `_meta/scripts/` and create `validate_manifest_completeness.md`: a specification for a script that cross-checks MANIFEST "What I Contain" tables against actual directory contents. Any file or folder that exists but isn't listed in MANIFEST → log to `pending.txt` as `shallow_node`. Any MANIFEST row pointing to a file that doesn't exist → log as `missing_composition`.
- Add to `runner.md` session-end checklist: "Run validate_manifest_completeness.py in addition to validate_manifests.py."

### BAD-07: CONTEXT.md Routing Has No Automated Link Validation
CONTEXT.md files are routing tables. Every row has a condition and a target. There is no automated check that every target path in every routing table resolves to a real file or folder. Broken routes are invisible until an agent tries to navigate and fails — which could be many sessions after the route broke.

**What to do:**
- Create `_meta/scripts/validate_context_routes.md`: specification for a script that reads every CONTEXT.md in the workspace, extracts all file path targets from routing tables, and verifies that each target exists. Dead routes → log to `pending.txt` as `missing_composition`, severity: `high`.
- Add to runner.md session-end checklist.
- Add gap to `_meta/gaps/pending.txt`: "No automated CONTEXT.md route validation exists. Broken routes invisible until runtime."

### BAD-08: REGISTRY.md Quality Tracking Requires Manual Updates
The REGISTRY.md quality tracking (avg_quality, run_count, last_run) requires stage-05-update to write manually after every run. All values currently show `—` and `0`. There's no telemetry infrastructure feeding this. The quality system exists on paper but generates no actual quality data.

**What to do:**
- Open `_registry/REGISTRY.md`
- Add a note in the Quality Update Protocol: "Until telemetry infrastructure is built, quality updates are manual. Session runner must update this table after any session that completes a workspace type run. Use session-close checklist in `_meta/runner.md`."
- Create a gap: `_meta/gaps/registry-telemetry.md` with type: `shallow_node`, description: "REGISTRY quality tracking is manual — no automated telemetry feeds it. avg_quality, run_count, last_run are all null across all task types despite multiple sessions completing.", severity: `medium`.

### BAD-09: leftOffHere.md Multi-Agent Merge Is a Naming Convention, Not a Protocol
The current leftOffHere.md has sections "(this agent)" and "(other agent)". Two agents contributed to the same session. Their states are merged into one file by informal section naming — there is no merge protocol. Two agents writing simultaneously would corrupt the file. There is no locking mechanism. There is no merge conflict resolution.

**What to do:**
- Create `leftOffHere-protocol.md` in workspace root: defines the multi-agent state merge rules: each agent writes a named snapshot to `_meta/session-snapshots/[agent-id]-[timestamp].md`. The human or coordinator agent runs `_meta/runner.md` merge step which synthesizes all snapshots into a single leftOffHere.md. No agent writes directly to leftOffHere.md.
- Add a gap: "leftOffHere.md has no multi-agent write protocol — concurrent writes would corrupt session state."

### BAD-10: No Rollback Mechanism If CONVENTIONS.md Gets Corrupted
CONVENTIONS.md is the single source of truth that WINS over all local files. If CONVENTIONS.md itself gets incorrectly edited — wrong pattern, deleted section, introduced contradiction — every agent in the system will follow the corrupted version, and the corruption will propagate into every local file update that follows. There is no documented rollback.

**What to do:**
- Open `_core/CONVENTIONS.md`
- Add a footer section: `## Integrity Protocol`: "CONVENTIONS.md is immutable mid-session. No agent may edit CONVENTIONS.md without a formal ADR proposing the change. The ADR must reference the pattern number being changed, the old definition, the new definition, and the reason. All local files referencing the changed pattern must be audited after the update."
- Create `_core/CONVENTIONS-changelog.md`: date-stamped log of every pattern version change.

### BAD-11: pending.txt Conflates Gap Objects AND Unclassified Documents
The intake pipeline spec says: when a document is classified as `question` or `unknown`, it "stops at Stage 03 and logs to pending.txt." But pending.txt is also where gap objects from all agents are logged. Now pending.txt is both a gap registry AND an unclassified document queue. These are different things handled by different agents.

**What to do:**
- Create `_meta/intake-pipeline/stage-03-route/unclassified-queue.md`: a dedicated file for documents that stopped at Stage 03. This is NOT the same as `_meta/gaps/pending.txt`.
- Update `_meta/intake-pipeline/stage-03-route/CONTEXT.md` to route unclassified documents to this new file.
- Add a gap: "pending.txt conflates two distinct concerns: gap objects and unclassified intake documents. Separate into two files."

### BAD-12: The Terry Davis Test Is Aspirational, Not Mechanically Enforceable
"Replace all domain-specific nouns with [DOMAIN NOUN] — does the file still work?" requires a human to make the judgment call on what counts as domain-specific and whether the replacement "still works." There is no mechanical version of this test. It's a discipline, not a constraint.

**What to do:**
- Accept this as a permanent limitation. Document it explicitly.
- Open `_core/CONVENTIONS.md §20`
- Add under the Test description: `## Known Limitation`: "The Terry Davis test requires human judgment at two points: (1) which nouns count as domain-specific, and (2) whether the post-replacement file 'still works.' There is no mechanical version of this test. Sessions must include at least one human Terry Davis test for every new root-level or _meta/ file created."

### BAD-13: Windows-Only Absolute Paths in leftOffHere.md
`leftOffHere.md` contains: `C:\Users\Quandale Dingle\yearTwo777\workspace-blueprint\workspace-blueprint\`. This is machine-specific. Backslashes. Specific username. Specific drive letter. If the workspace is moved to another machine, another user account, or a Unix system, every path in every document that references the workspace root breaks.

**What to do:**
- Add a `## Path Abstraction Rule` to `_meta/runner.md`: "Never write absolute paths in leftOffHere.md. Write workspace-relative paths only (e.g., `programs/game_engine/world/...`). The workspace root is always the directory containing the root MANIFEST.md."
- Update the current leftOffHere.md to replace all absolute paths with workspace-relative paths.
- Add to CONVENTIONS.md P-25 (New Folder Protocol): "File references must use workspace-relative paths. Absolute paths are forbidden in all markdown files."

### BAD-14: campaigns/ Is Not in programs/_template/
The campaign pattern is a first-class concept — two campaigns already exist, a third being created now. But `programs/_template/` does not include a `campaigns/` folder. Every project cloned from _template must manually discover and create its own campaigns folder. This breaks the "clone template, you're done" promise.

**What to do:**
- Add `campaigns/` to `programs/_template/` with a `README.md`: "This folder contains campaign archives. Each campaign = one folder named after the campaign with prompt.md, clarification.md, and progress/."
- Add to `_template/MANIFEST.md` "What I Contain" table: `campaigns/ | folder | archive of user requirements, agent interpretations, and progress documents for this project`
- Update `_core/CONVENTIONS.md` to add P-29: Campaign Protocol — campaigns/ is standard in every project, created during project scaffold.

### BAD-15: P-23 ADR Logic Would Grind All Sessions to a Halt If Strictly Applied
"If a decision was not explicit in the PRD, it is an assumption. Assumption-status ADRs block building." Every PRD leaves hundreds of implementation details undecided. If every unspecified detail becomes a blocking assumption, no agent can make progress without constant human validation. The spec acknowledges this implicitly (agents ARE making decisions and logging them as `accepted`) but the protocol has no threshold for what "explicit in the PRD" means.

**What to do:**
- Open `_core/CONVENTIONS.md §23 (P-23)`
- Add `## Threshold Clarification`: "An `assumption` is only blocking when it affects a system BOUNDARY (network protocol, database schema, shared contract, public API shape). Internal implementation details that do not cross system boundaries may be logged as `assumption` but do NOT block building. The rule of thumb: if changing this decision later would require a contract change or interface change, it's a blocking assumption. If it would require only code changes internal to one program, it's a non-blocking assumption."

### BAD-16: CONVENTIONS.md Patterns Have No Version Numbers
P-14 today could mean something different than P-14 in three months if it's edited. Every local file that references P-14 has no way to know which version of P-14 it was referencing when written. Pattern drift is invisible.

**What to do:**
- Add a `version` field to every pattern in CONVENTIONS.md. Format: `P-14 v1.0` in the index.
- Create `_core/CONVENTIONS-changelog.md` (as also noted in BAD-10) with dated entries for every pattern change.
- Add rule to the CONVENTIONS.md preamble: "When changing a pattern, increment its version. When v1.0 → v2.0, search all local files for references to the old pattern number and audit whether they need updating."

### BAD-17: The 5D Semantic Vector Has No Calibration Guide
`specificity: 0 (abstract) → 1 (concrete)`. This sounds precise. But two people placing the same document on this scale will get different results because "abstract" and "concrete" are subjective without calibration. There are no anchor examples. No test documents with known coordinates. No disambiguation guide.

**What to do:**
- Open `programs/knowledge-graph/_planning/`
- Create `5d-vector-calibration.md`: For each of the five dimensions, provide 3–5 anchor examples that define the low, middle, and high positions. Example for specificity: `0.1 = "consciousness exists" (philosophy paper abstract), 0.5 = "use React for UI" (tech decision), 0.9 = "set Redis TTL to 3600 seconds" (concrete config value)`. These anchors enable consistent placement.

### BAD-18: Fix-First Rule Has No Scope Boundary — Can Eat Entire Sessions
P-22: "When an agent identifies an error, fix it immediately." In a large workspace, one error fix exposes another. That fix exposes a third. Following P-22 strictly, an agent that enters to do Task A could spend the entire session fixing discovered errors and never touch Task A. There is no scope limit.

**What to do:**
- Open `_core/CONVENTIONS.md §22 (P-22)`
- Add `## Scope Boundary`: "Fix-First applies within the SCOPE of the current session task. An error discovered in a folder OUTSIDE the current task's scope is logged to `_meta/gaps/pending.txt` with `severity: medium` and deferred — not fixed immediately. An error discovered WITHIN the current task's scope is fixed immediately before continuing. The session runner defines scope at session start."
- Update `_meta/runner.md` to include session scope declaration as step 1.

### BAD-19: No Deprecation Cleanup Protocol
`project-alpha/` and `project-beta/` exist at the workspace root and are marked deprecated in CLAUDE.md. They've been there since before `programs/project-alpha/` and `programs/project-beta/` superseded them. There is no protocol for physically removing deprecated content — or deciding when it's safe to. Deprecated content accumulates.

**What to do:**
- Create `_meta/deprecation-protocol.md`: Three-stage deprecation:
  - Stage 1 (mark): Status → `deprecated` in MANIFEST.md, note what supersedes it.
  - Stage 2 (archive): After 30 days with no reads, move to `_archive/` at root level.
  - Stage 3 (delete): After 90 days in `_archive/` with no reads, delete with a final log entry in `_meta/gaps/deprecation-log.md`.
- Immediately move `project-alpha/` and `project-beta/` from root into `_archive/` per Stage 2 (they've been superseded).

### BAD-20: spec-review.md Gate Doesn't Define Who Runs It — Circular Self-Review
spec-review.md is the only gate from scaffold → specced. But who runs it? In practice, the same agent that built the spec runs spec-review.md against the spec it just built. This is self-review. An agent that wrote a broken spec will still pass its own spec-review because it believes its own reasoning.

**What to do:**
- Open `_meta/spec-review.md`
- Add `## Reviewer Protocol`: "Spec review must be run by a SEPARATE session from the one that built the spec. The reviewing agent must load only the spec artifacts and spec-review.md — it must NOT have the building session's context. If the same agent runs spec-review immediately after building, the review is invalid. At minimum, spec review should be a fresh session."
- Add this to the quality gate description in `_core/CONVENTIONS.md §24 (P-24)`.

### BAD-21: "New folder" and "New Text Document.txt" in Workspace Root — P-25 Violation
There are two Windows filesystem artifacts at the workspace root: `New folder/` (no MANIFEST.md) and `New Text Document.txt` (no MANIFEST entry, unknown purpose). P-25 requires every folder to have a MANIFEST.md. P-22 says identify and fix. These have been sitting there.

**What to do:**
- Immediately: delete `New Text Document.txt` (zero content, Windows artifact) and `New folder/` if empty.
- If `New folder/` has content, read its contents, create a MANIFEST.md describing what it contains, then route it to the right location.
- Log the fix to `_meta/gaps/pending.txt`.
- Add a note to `_meta/runner.md` session-start checklist: "Verify no unnamed Windows artifacts exist in workspace root."

### BAD-22: The Workspace Has an Unstated Windows-Only Assumption
The Terry Davis Constraint says the system must work for any domain. But the workspace itself uses Windows path conventions, Windows-generated files (`New Text Document.txt`), backslash paths, and drive letters. The system violates its own principle at the OS level.

**What to do:**
- Add to `START-HERE.md` a `## Platform Assumptions` section that honestly states: "This workspace was developed on Windows 11. Path examples use Windows conventions. On Unix/Linux/Mac: replace backslashes with forward slashes, remove drive letters, use shell-appropriate equivalents."
- Add to `_core/CONVENTIONS.md` a note on P-20: "The Terry Davis Constraint applies to DOMAIN knowledge abstraction. It does not require OS-independence. Platform requirements should be documented explicitly rather than hidden."

### BAD-23: claudehist3-13-2026.txt Is an Unmanaged Information Leak
A raw Claude conversation history file sits in the workspace root. It has no MANIFEST entry. It has no status. It was not run through the intake pipeline. It contains the full context of a prior session, including potentially sensitive information about the user, their projects, their decision-making process, and their work. It's also noise that will confuse agents navigating the root.

**What to do:**
- Move `claudehist3-13-2026.txt` to `_archive/session-histories/claudehist3-13-2026.txt`
- Add `_archive/` to root MANIFEST.md "What I Contain" with type: `folder`, purpose: `long-term archive for deprecated content and session histories`
- Add a runner.md session-start note: "Session history files (.txt) must not live in workspace root. Route to _archive/session-histories/."

### BAD-24: "exmple.md" Typo Filename — Invisible to Agent Navigation
There is a file named `exmple.md` (not `example.md`) at the workspace root. It has no MANIFEST entry. P-22 says identify and fix. This file has been sitting with a misspelled name and no routing information. It is invisible to any agent navigating via MANIFEST routing.

**What to do:**
- Read `exmple.md` to determine its content and purpose.
- Rename to the correct name based on content.
- Add it to the root MANIFEST.md "What I Contain" table.
- Log the fix to `_meta/gaps/pending.txt`.

### BAD-25: REGISTRY.md Shows game-engine as "scaffold" — Contradicts leftOffHere.md "specced"
`_registry/REGISTRY.md` shows: `game-engine | programs/game_engine/ | scaffold`. But `leftOffHere.md` shows all 4 Phase 0 sub-programs passed spec-review on 2026-03-14 and the project is specced and ready to build. Two sources of truth disagree about the project status. This is exactly the failure mode that "status lives ONLY in MANIFEST.md" is designed to prevent — but the REGISTRY is also claiming to know status.

**What to do:**
- Update `_registry/REGISTRY.md` game-engine row: `status: specced` (reflects actual state per leftOffHere.md and MANIFEST.md)
- Add oracle row: `oracle | programs/oracle/ | specced | — | 0 | —` (oracle reached specced this session)
- Add to REGISTRY.md protocol: "REGISTRY status must mirror project MANIFEST.md status. MANIFEST is canonical. REGISTRY is a routing index — it must not diverge. Session runner updates REGISTRY status at close using MANIFEST as source."
- Add gap: "REGISTRY.md status tracking has no sync mechanism with MANIFEST.md status. Manual updates required — will drift."

---

## 4. Priority Order for Execution

Gaps that can be fixed by an agent with no human input, in priority order:

| Priority | Bad Item | Action | Effort |
|----------|----------|--------|--------|
| 1 | BAD-21 | Delete Windows artifacts from root | 5 min |
| 2 | BAD-23 | Move claudehist to _archive/ | 5 min |
| 3 | BAD-24 | Read + rename exmple.md | 10 min |
| 4 | BAD-25 | Update REGISTRY.md status fields | 10 min |
| 5 | BAD-13 | Remove absolute paths from leftOffHere.md | 15 min |
| 6 | BAD-02 | Add root CLAUDE.md exception to P-20 | 20 min |
| 7 | BAD-05 | Add Pattern Tiers to CONVENTIONS.md | 20 min |
| 8 | BAD-14 | Add campaigns/ to _template/ | 20 min |
| 9 | BAD-11 | Create unclassified-queue.md, separate from pending.txt | 25 min |
| 10 | BAD-19 | Create deprecation-protocol.md, move root project-alpha/beta | 30 min |
| 11 | BAD-15 | Clarify blocking vs non-blocking assumption threshold in P-23 | 20 min |
| 12 | BAD-18 | Add scope boundary to Fix-First rule P-22 | 20 min |
| 13 | BAD-10 | Add CONVENTIONS.md integrity protocol + changelog | 30 min |
| 14 | BAD-16 | Add version numbers to all 28 patterns | 30 min |
| 15 | BAD-07 | Write validate_context_routes.md spec | 30 min |
| 16 | BAD-06 | Write validate_manifest_completeness.md spec | 30 min |
| 17 | BAD-08 | Add manual update reminder to runner.md for REGISTRY | 20 min |
| 18 | BAD-09 | Create leftOffHere-protocol.md for multi-agent sessions | 30 min |
| 19 | BAD-03 | Add mid-session checkpoint protocol to runner.md | 20 min |
| 20 | BAD-04 | Add severity tiebreaker protocol to gap-detection-agent.md | 20 min |
| 21 | BAD-17 | Create 5d-vector-calibration.md for knowledge-graph | 45 min |
| 22 | BAD-20 | Add reviewer protocol to spec-review.md | 20 min |
| 23 | BAD-12 | Document Terry Davis mechanical limitation in P-20 | 15 min |
| 24 | BAD-22 | Add Platform Assumptions section to START-HERE.md | 15 min |
| 25 | BAD-01 | Create BUILD-FIRST gap + update leftOffHere.md | 10 min |

Gaps requiring human judgment before acting:
- BAD-25 (oracle REGISTRY row) — verify oracle status is correctly specced
- BAD-21 (New folder) — confirm New folder/ is safe to delete

---

## 5. What Was Completed This Campaign (running log)

| Date | Item | Status |
|------|------|--------|
| 2026-03-16 | Campaign folder created | DONE |
| 2026-03-16 | prompt.md written | DONE |
| 2026-03-16 | clarification.md written | DONE |
| 2026-03-16 | progress/perfect_final_output.md written | DONE |

---

## 6. Admin Notes

- **2026-03-16**: Campaign created. 25 good / 25 bad analysis complete. 25 action items mapped to folder architecture. Priority order established. No code produced — this campaign is documentation and gap creation only. Build work goes through the individual project campaigns, not this one.
- **Future sessions**: Execute actions in priority order from Section 4. Mark rows complete in Section 5. This clarification doc is the single source of truth for campaign progress.
