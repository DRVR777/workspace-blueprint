# PRD INTAKE — Read a PRD and Build the Project

You are given a PRD document. Your job is to read it, understand it,
and build a complete project scaffold inside this workspace.

Do not ask for clarification. Read the PRD, make decisions, log every
inference to pending.txt, and build. The gap system will surface anything
that needs human review.

---

## TRIGGER RECOGNITION

This agent is invoked by any of:
1. User types `intake: "[prd text]"` — treat the quoted content as the PRD
2. A file exists in `_intake/queue/` — read it as the PRD
3. A file is passed directly to this session
4. The user pastes a PRD into the conversation

All four paths lead to the same process below. No path requires the user
to touch a folder or run a command.

---

## PHASE 0: READ THE PRD

Find the PRD. Check in order:
1. `intake:` trigger text in the current conversation (quoted after the colon)
2. A file passed directly to this session
3. `_intake/queue/` — any .md, .txt, or .pdf file present
4. Content pasted into this conversation

Read the entire PRD before touching any files.

Extract and hold in working memory:

```
PROJECT_NAME:      [derive from PRD title or product name — lower-case-hyphenated]
ONE_LINE:          [one sentence: what problem it solves and what it produces]
PROGRAMS:          [list every distinct runnable/deployable component]
SHARED_CONTRACTS:  [list every interface between programs]
DECISIONS:         [list every architectural choice stated or implied in the PRD]
UNKNOWNS:          [list everything the PRD doesn't answer that a builder will need to know]
```

If you cannot extract PROJECT_NAME → use the filename slug.
If you cannot extract PROGRAMS → that is itself a gap. Log it. Scaffold one program called `core/`.

---

## PHASE 1: CHECK IF THE PROJECT EXISTS

Does `[PROJECT_NAME]/` exist at workspace root?

Projects live in `programs/[PROJECT_NAME]/`. Check there first.

**If YES (programs/[PROJECT_NAME]/ exists):**
  Read `programs/[PROJECT_NAME]/MANIFEST.md`. Check the `status` field.
  - `status = scaffold` → fresh scaffold, unclaimed. Proceed to Phase 2.
  - `status = active` → real code exists. Stop.
    Log to root `_meta/gaps/pending.txt`:
    ```
    [timestamp] | root | CLAUDE.md | inferred "PRD for [PROJECT_NAME] received but project already exists as active — may be update or conflict" — no file states this
    ```
    Report: "Project [PROJECT_NAME] already exists with status: active.
    Use the UPDATE prompt instead. To force re-scaffold: re-run with FORCE_SCAFFOLD=true."
    Stop.

**If NO:** Run the template clone first (before Phase 2):
  1. If `_meta/scripts/new_project.py` is available: `python _meta/scripts/new_project.py [PROJECT_NAME]`
  2. Otherwise: manually copy `programs/_template/` to `programs/[PROJECT_NAME]/`
     and replace all `{{PROJECT_NAME}}` placeholders with the actual project name.
  The template provides the correct empty structure. Phase 3 populates it.
  Proceed to Phase 2.

---

## PHASE 2: LOG ALL INFERENCES BEFORE BUILDING

Before creating any files, write every inference to root `_meta/gaps/pending.txt`.

Required entries:

```
[timestamp] | [PROJECT_NAME]/_planning | PRD | inferred "project [PROJECT_NAME] does not exist — scaffolding from PRD" — PRD is the source, not an existing architecture file
[timestamp] | [PROJECT_NAME]/_planning | PRD | inferred "programs are: [PROGRAMS list]" — derived from PRD description
[timestamp] | [PROJECT_NAME]/shared | PRD | inferred "contracts needed: [SHARED_CONTRACTS list]" — derived from PRD description
```

For every item in UNKNOWNS:
```
[timestamp] | [PROJECT_NAME]/_planning | PRD | inferred "[assumption made to fill this unknown]" — PRD does not state this
```

---

## PHASE 3: POPULATE THE PROJECT SCAFFOLD

The template clone already exists at `programs/[PROJECT_NAME]/` (from Phase 1).
Populate it. Do not recreate structure from scratch — fill in the placeholders.
Build in this exact order. Do not skip. Do not reorder.

All files go to `programs/[PROJECT_NAME]/`, not to the workspace root.

### 3.1 — Project root

**`[PROJECT_NAME]/MANIFEST.md`:**
```markdown
# MANIFEST — [PROJECT_NAME]

## Envelope
| Field | Value |
|-------|-------|
| `id` | [PROJECT_NAME] |
| `type` | project |
| `depth` | 1 |
| `parent` | workspace root |
| `version` | 0.1.0 |
| `status` | scaffold |
| `prd_source` | [filename or "inline"] |
| `created` | [ISO-8601 timestamp] |

## What I Am
[ONE_LINE]

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| CLAUDE.md | file | active | Depth-1 map of this project's programs |
| CONTEXT.md | file | active | Task router |
| _planning/ | folder | active | Architecture decisions before code |
| _meta/ | folder | active | Project-internal gap registry |
| shared/ | folder | active | Contracts and types between programs |
| programs/ | folder | active | All runnable/deployable components |
[one row per program from PROGRAMS]

## What I Need From Parent
Nothing — self-contained. Cross-project deps in root _meta/gaps/.

## What I Return To Parent
[The project's output: an API, a CLI, a deployed service, etc.]

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Make an architectural decision | _planning/CONTEXT.md |
| Work inside a specific program | programs/[name]/CONTEXT.md |
| Define or update a contract | shared/MANIFEST.md |
| Log a gap | _meta/gaps/pending.txt |
| Orient with no prior context | This MANIFEST, then CLAUDE.md |

## Gap Status
Gaps from PRD intake logged at creation. See _meta/gaps/CONTEXT.md.
```

**`[PROJECT_NAME]/CLAUDE.md`:**
```markdown
<!-- See _meta/ur-prompt.md before reading this file. -->
# [PROJECT_NAME] — Project Map

## Hard Rule
Depth 1 only. Program names and one-line purposes.
Internal structure of each program lives in that program's own CONTEXT.md.

---

## Programs

| Program | Purpose | Status |
|---------|---------|--------|
[one row per program — purpose from PRD, status = scaffold]

---

## Workspace Rules

1. An agent in one program never loads another program's src/.
2. All cross-program data shapes live in shared/contracts/.
3. Check _planning/adr/ before writing code — decisions may already be made.
4. Log every inference to _meta/gaps/pending.txt during the task, not after.

---

## Navigation

| You want to... | Go to |
|----------------|-------|
| Plan or decide architecture | _planning/CONTEXT.md |
| Work on a specific program | programs/[name]/CONTEXT.md |
| Find a contract | shared/contracts/ |
| See open gaps | _meta/gaps/CONTEXT.md |
| See all architectural decisions | _planning/adr/ |
| Look up an architectural pattern | {root}/_core/CONVENTIONS.md |

---

## What to Load

Minimum file set per task type. Do not load output/ unless continuing in-progress work.

| Task | Load these files | Do NOT load |
|------|-----------------|-------------|
| Start a new program | MANIFEST.md, _planning/CONTEXT.md, _planning/prd-source.md | other programs' files |
| Work on an existing program | programs/[name]/CONTEXT.md, programs/[name]/MANIFEST.md, shared/contracts/ | _planning/prd-source.md, other programs |
| Review architecture decision | _planning/adr/[relevant ADR] | program source files |
| Run spec review | programs/[name]/CONTEXT.md, programs/[name]/MANIFEST.md | output/ folders |
| Close a gap | _meta/gaps/CONTEXT.md, the gap JSON file | unrelated program files |
| Find a contract | shared/MANIFEST.md, shared/contracts/[name].md | program source files |
```

**`[PROJECT_NAME]/CONTEXT.md`:**
```markdown
# [PROJECT_NAME] — Task Router

## What This Project Is
[ONE_LINE]

---

## Task Routing

| Your Task | Go To | Also Load |
|-----------|-------|-----------|
| Plan architecture or make a technical decision | _planning/CONTEXT.md | Nothing else yet |
[one row per program: "Work on [program]" | programs/[name]/CONTEXT.md | shared/contracts/[relevant]]
| Define or update a contract | shared/MANIFEST.md | _planning/adr/ for context |
| Log an inference or gap | _meta/gaps/pending.txt | Nothing |

---

## Status
Scaffolded from PRD. Programs are stubs.
Work in _planning/ before writing any code in programs/.
```

---

### 3.2 — Planning workspace

**`[PROJECT_NAME]/_planning/CONTEXT.md`:**
```markdown
# _planning — Architecture Workspace

## What This Is
Where decisions get made before code gets written.
Do not create files in programs/ without checking whether a decision exists here first.

---

## What Lives Here

| Folder/File | Purpose |
|-------------|---------|
| adr/ | Architecture Decision Records — permanent, never deleted, only superseded |
| system-design/ | Data flows, service boundaries, component diagrams |
| roadmap.md | Build order — what gets built first and why |

---

## Task Routing

| Your Task | Do This |
|-----------|---------|
| Make a new decision | Check adr/ first — if it exists, read it. If not, write a new ADR. |
| Document system design | Create or update files in system-design/ |
| Check project status | Read roadmap.md |

---

## ADR Format

File: `adr/[NNN]-[decision-slug].md`

Status values:
- `proposed` — written but not yet validated
- `accepted` — binding for all programs in this project
- `assumption` — inferred to fill a PRD gap — needs human validation before the affected program builds
- `superseded by ADR-[NNN]` — replaced, never deleted

An agent encountering an `assumption` ADR must not build the affected feature until
a human changes the status to `accepted`.

---

## What Planning Produces

| Output | Goes To |
|--------|---------|
| Accepted ADRs | Binding constraints for programs/*/CONTEXT.md |
| System design | Reference for build agents |
| Contract definitions | ../shared/contracts/ (planning defines, shared/ holds) |
| Roadmap | Build order for the whole project |
```

**`[PROJECT_NAME]/_planning/adr/README.md`:**
```markdown
# Architecture Decision Records

One file per decision. Format: [NNN]-[slug].md
Number sequentially from 001. Never delete. Supersede instead.

Status legend:
- accepted: binding, derived from PRD explicit statements
- assumption: inferred to fill PRD gap — needs human validation
- proposed: written but not yet reviewed
- superseded by ADR-NNN: replaced
```

For each item in DECISIONS — write `[PROJECT_NAME]/_planning/adr/[NNN]-[slug].md`:
```markdown
# ADR [NNN]: [Decision Title]

## Status
accepted — stated or clearly implied in PRD

## Context
[What the PRD stated that led to this decision]

## Decision
[The decision, stated precisely]

## Consequences
[What this means for the build — complete during planning phase]

## Alternatives Considered
[To be completed during planning phase]
```

For each item in UNKNOWNS — write `[PROJECT_NAME]/_planning/adr/[NNN]-[slug].md`:
```markdown
# ADR [NNN]: [Assumption Title]

## Status
assumption — PRD did not specify this; agent inferred the following

## Context
[What the PRD left unclear]

## Decision
[What was assumed to fill the gap]

## Consequences
TBD — validate before building the affected program.

## Alternatives Considered
Unknown — this needs human review before status changes to accepted.
```

---

### 3.3 — Project _meta (gap registry)

**`[PROJECT_NAME]/_meta/gaps/pending.txt`:**
```
# INFERENCE LOG — [PROJECT_NAME] scope
# Format: [ISO-8601-timestamp] | [program] | [file-consulted] | inferred "[what]" — no file states this
# Cross-project inferences → root _meta/gaps/pending.txt instead
# Do not edit existing entries. Only append.

```

For each item in UNKNOWNS, write `[PROJECT_NAME]/_meta/gaps/gap-[NNN]-[slug].json`:
```json
{
  "gap_id": "gap-[NNN]",
  "detected_at": "[ISO-8601 timestamp]",
  "detected_in": "[PROJECT_NAME]/_planning",
  "gap_type": "missing_composition",
  "scope": "[PROJECT_NAME]",
  "description": "[What the PRD left unspecified]",
  "evidence": "PRD intake: this was not stated in the source document",
  "severity": "blocking",
  "proposed_expansion": "Write an accepted ADR in _planning/adr/ resolving this unknown before building the affected program",
  "status": "open",
  "closed_by": null
}
```

**`[PROJECT_NAME]/_meta/gaps/CONTEXT.md`:**
```markdown
# Gap Registry — [PROJECT_NAME]

## Scope: [PROJECT_NAME]-internal only
Cross-project gaps go to root _meta/gaps/.

## Current Open Gaps

| Gap ID | Type | Description | Severity | Status |
|--------|------|-------------|----------|--------|
[one row per UNKNOWN — all blocking until resolved]

## Resolution Protocol

For blocking gaps (PRD unknowns):
Do not write code in the affected program until the gap is closed.
Close by: writing an ADR with status `accepted` in _planning/adr/ and updating gap status here.

For degrading/cosmetic gaps:
Follow the standard session protocol in root _meta/runner.md.

## Escalation Rule
If a gap involves any folder outside [PROJECT_NAME]/ → root _meta/gaps/pending.txt instead.
```

---

### 3.4 — Shared contracts

**`[PROJECT_NAME]/shared/MANIFEST.md`:**
```markdown
# MANIFEST — [PROJECT_NAME]/shared

## Envelope
| Field | Value |
|-------|-------|
| `id` | [PROJECT_NAME]-shared |
| `type` | contracts |
| `depth` | 2 |
| `parent` | [PROJECT_NAME]/ |
| `status` | active |

## What I Am
The hard boundary between programs. Programs never import from each other.
If two programs need to agree on a data shape — the definition lives here.

## The Rule
If it's not defined here, both sides will infer independently → divergence → missing_bridge gap.

## Contracts Needed (from PRD)

| Contract | Produced By | Consumed By | Status |
|----------|-------------|-------------|--------|
[one row per item in SHARED_CONTRACTS]
```

For each item in SHARED_CONTRACTS, write `[PROJECT_NAME]/shared/contracts/[contract-slug].md`:
```markdown
# Contract: [Contract Name]

## Status
stub — shape not yet defined. Do not build against this until shape is defined.

## Produced By
[program-name]

## Consumed By
[program-name]

## Shape
[Define the exact data shape here before either program builds against it]
[Fields, types, required vs optional, example values]
```

---

### 3.5 — Program scaffolds

For each program in PROGRAMS:

**`[PROJECT_NAME]/programs/[program-name]/MANIFEST.md`:**
```markdown
# MANIFEST — programs/[program-name]

## Envelope
| Field | Value |
|-------|-------|
| `id` | [PROJECT_NAME]-programs-[program-name] |
| `type` | program |
| `depth` | 3 |
| `parent` | [PROJECT_NAME]/programs/ |
| `status` | scaffold |

## What I Am
[What this program does and what it exposes — derived from PRD]

## External Dependencies
| Depends On | What | Contract Location |
|------------|------|-------------------|
[derive from PRD and SHARED_CONTRACTS — if none, write "none"]

## What I Produce
[Output: an API, a built binary, a rendered UI, etc.]

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Write a new feature | CONTEXT.md → load relevant contract from ../../shared/contracts/ |
| Architecture question | ../../_planning/CONTEXT.md |
| Define a new inter-program interface | ../../shared/contracts/ |

## Gap Status
See ../../_meta/gaps/CONTEXT.md — check for blocking gaps before building.
```

**`[PROJECT_NAME]/programs/[program-name]/CONTEXT.md`:**
```markdown
# programs/[program-name] — Task Router

## What This Program Is
[One sentence from PRD]

---

## Before Writing Any Code

These are hard stops. Not suggestions.

**Stop 1 — Assumption ADR check:**
Read every file in `../../_planning/adr/`.
If any ADR has `status: assumption` and its Decision or Consequences mentions this program → STOP.
Do not write a single line of code. Surface the ADR to a human for validation.
A human must change the status to `accepted`. Then re-check.

**Stop 2 — Stub contract check:**
Read every contract listed in this program's MANIFEST.md `External Dependencies` table.
If any contract at `../../shared/contracts/[contract]` has `status: stub` → STOP.
Define the contract shape before writing code that produces or consumes it.
A stub is a placeholder. Building against a placeholder produces code that will break when the real shape is defined.

**Stop 3 — Spec review:**
Run `../../_meta/spec-review.md` on this program.
If verdict is not `OVERALL: PASS` → STOP.
Fix every blocking item listed in the verdict. Re-run spec-review. Repeat until PASS.
Only then: change this program's MANIFEST.md `status` from `scaffold` to `specced` and begin building.

See `../../_meta/status-transitions.md` for the complete gate criteria.

---

## Task Routing

| Your Task | Load These | Skip These |
|-----------|-----------|------------|
| Write a new feature | MANIFEST.md, relevant contract | Other programs' src/ |
| Fix a bug | The failing file, its contract | _planning/ (unless fix changes an interface) |
| Write tests | Code under test, its contract | Everything else |
| Update an interface | ../../shared/contracts/[contract], ../../_planning/adr/ | src/ until contract updated |

---

## Checkpoints

Pause at each checkpoint. Present the unit of work. Wait for human steering before proceeding.

| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Contract review | All contracts for this program with proposed shapes | approve / revise shapes / add contracts |
| First feature complete | The feature's output + which contract(s) it satisfies | approve / revise / restart |
| All features complete | Summary of outputs vs PRD requirements | approve for Audit / revise specific feature |

---

## Audit

Run this checklist before writing anything to output/. All items must pass.

- [ ] Every function/module references a contract defined in `../../shared/contracts/`
- [ ] No values from another program's src/ are imported directly
- [ ] Every assumption made during this build is logged in `../../_meta/gaps/pending.txt`
- [ ] No ADR with `status: assumption` was built against without human validation
- [ ] Output matches the shape defined in MANIFEST.md `What I Produce`
- [ ] (Content programs only) Output hits ≥ 2 of: NOVEL / USABLE / QUESTION-GENERATING / INTERESTING — see _core/CONVENTIONS.md §14
- [ ] (Content programs only) Output passes all checks in `../../shared/voice.md`

If any item fails: fix in place, re-run the failing item, log the repair to pending.txt.

---

## Inference Logging
All inferences → `../../_meta/gaps/pending.txt`
Format: `[timestamp] | [program-name] | [file] | inferred "[what]" — no file states this`
```

---

### 3.6 — Update root CLAUDE.md

Add the new project to the Projects table in root `CLAUDE.md`.

If a placeholder row exists (project-alpha, project-beta) in `programs/`, replace the first unclaimed one.
If no placeholder exists, append to the Projects table:
```markdown
| `programs/[PROJECT_NAME]/` | [ONE_LINE] | scaffold |
```

---

### 3.7 — Archive the PRD

If PRD came from `_intake/queue/`:
Move it to `_intake/processed/[PROJECT_NAME]-prd-[ISO-date].[ext]`

If PRD was pasted inline:
Write it to `[PROJECT_NAME]/_planning/prd-source.md`

This file is the permanent record of what initiated the scaffold. Never delete it.

---

## PHASE 4: REPORT

```
PRD INTAKE REPORT
=================

Project:     [PROJECT_NAME]
Created:     [ISO-8601 timestamp]
Location:    [PROJECT_NAME]/

FILES CREATED
-------------
[list every file created with path]

PROGRAMS SCAFFOLDED
-------------------
[program-name] — [one-line purpose]

CONTRACTS STUBBED
-----------------
[contract-slug] — needs shape definition before programs build against it

ADRs WRITTEN
------------
[NNN]-[slug] — accepted (from PRD)
[NNN]-[slug] — assumption (inferred — needs human validation)

OPEN BLOCKING GAPS
------------------
These must be closed before code is written in the affected programs.

| Gap ID | Description | Close By |
|--------|-------------|----------|
[one row per UNKNOWN]

NEXT ACTIONS (in order)
-----------------------
1. Open [PROJECT_NAME]/_planning/adr/ — validate every "assumption" ADR.
   Change status to "accepted" or rewrite. Do not start building until this is done.

2. Open [PROJECT_NAME]/shared/contracts/ — define the shape of every stub.
   Programs cannot build against a stub.

3. Run gap detection: _meta/gap-detection-agent.md with scope: [PROJECT_NAME]
   This classifies all intake inferences into formal gap objects.

4. When all blocking gaps are closed and all assumption ADRs are accepted:
   run `_meta/spec-review.md` on each program before writing code.
   Build order: dependency graph first (see _planning/roadmap.md).
   No program builds until its spec-review verdict is OVERALL: PASS.

INFERENCE LOG ENTRIES WRITTEN: [n]
Root pending.txt: [n entries]
[PROJECT_NAME] pending.txt: [n entries]
```

---

## THE ONE RULE

Every time you derive something from the PRD that the PRD does not explicitly state,
log it to the appropriate `pending.txt` before writing the file that uses that derivation.

If `pending.txt` has zero entries after this run — either the PRD was perfectly complete
(impossible) or inference logging failed (log that as a gap in itself).
