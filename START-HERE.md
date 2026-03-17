# Workspace Blueprint — How to Read This Template

## What This Is

A template for building **agent-native workspaces** — folder structures designed so AI agents
can drop in, understand the work, and produce consistent output without being re-explained
context every session.

It's not a file organization system. It's a **context delivery system with enforced quality gates.**

---

## The Architecture

### Layer 0 — Meta Infrastructure (`_meta/`)
Domain-agnostic. Works for any type of project.
Contains the prompt generator, gap system, session protocol, PRD intake, and quality gates.
Never touches project content.

### Layer 1 — Project (`[project]/`)
A fully self-contained folder. One project per folder.
Has its own architecture decisions, programs, contracts, and gap registry.
Projects never share source code.

### Layer 2 — Program (`[project]/programs/[name]/`)
A single runnable/deployable component.
Has its own MANIFEST, CONTEXT, src/, and tests/.
Programs never import from each other — all shared shapes go through `shared/contracts/`.

---

## The 3-File Navigation System

Every folder that has routing decisions contains three files:

```
MANIFEST.md    Envelope: what this folder is, what it contains, what it needs,
               what it returns, gap status.

CLAUDE.md      Always-loaded map: depth-1 only. Names and one-line purposes.
               No internals. An agent reads this to know what exists.

CONTEXT.md     Task router: "what's your task → go here."
               The agent's entry point for doing work.
```

---

## The Quality Gate System

Nothing builds until it passes the gate.

```
PRD arrives
    ↓
prd-intake.md scaffolds the project
    ↓
Assumption ADRs created for every PRD unknown
    ↓
Human validates each assumption → changes status to "accepted"
    ↓
Stub contracts defined in shared/contracts/
    ↓
spec-review.md runs 4 checks (ADR gate, contract gate, coherence, gap gate)
    ↓
OVERALL: PASS → program moves scaffold → specced → build begins
```

See `_meta/spec-review.md` for the 4 checks.
See `_meta/status-transitions.md` for every gate's exact criteria.

---

## The Gap System (Closed Loop)

The system surfaces its own unknowns and converts them into tasks.

```
Agent runs → infers something not in any file
    ↓
Logs to [project]/_meta/gaps/pending.txt (or root _meta/gaps/ if cross-project)
    ↓
_meta/gap-detection-agent.md classifies entries into formal gap objects
    ↓
_meta/gaps/CONTEXT.md ranks open gaps by severity
    ↓
Highest-severity gap = next session's starting task
    ↓
_meta/ur-prompt.md generates an execution prompt from the gap object
    ↓
Agent runs → cycle repeats
```

---

## The Status Model

```
PROGRAM:  scaffold → specced → active → complete
PROJECT:  scaffold →           active → complete
```

- `scaffold` → plan not validated
- `specced` → spec-review PASS, ready to build
- `active` → code exists
- `complete` → all acceptance criteria met, tests pass

Status lives in each MANIFEST.md `status` field. Nowhere else.

---

## How to Use This Template

### Step 1 — Start with `_meta/`
Read `_meta/ur-prompt.md`. This is Layer 0. It generates prompts for any task from gap objects.
Read `_meta/runner.md`. This is the session protocol — how every session starts and ends.

### Step 2 — Drop in a PRD
Put a PRD document in `_intake/queue/`.
Run `_meta/prd-intake.md`. It scaffolds a complete project: MANIFEST, CLAUDE.md, CONTEXT.md,
_planning with ADRs, shared contracts (stubbed), programs (stubbed), gap registry.

### Step 3 — Resolve blockers before building
Open `[project]/_planning/adr/`. Find every ADR with `status: assumption`. Validate each one.
Open `[project]/shared/contracts/`. Define the shape of every stub contract.

### Step 4 — Run spec review
Run `_meta/spec-review.md` on each program in dependency order (see `_planning/roadmap.md`).
A program only builds after `OVERALL: PASS`.

### Step 5 — Build
With spec review passing, the program moves to `specced`. Build tasks come from the CONTEXT.md
task routing table. Every inference during build goes to pending.txt immediately.

### Step 6 — End every session properly
Run `_meta/gap-detection-agent.md`. Update gap statuses. The highest-severity open gap is
the starting point for the next session.

---

## File Structure

```
workspace-blueprint/
├── START-HERE.md              ← You are here
├── MANIFEST.md                ← Root envelope
├── CLAUDE.md                  ← Depth-1 map: project names and purposes
├── CONTEXT.md                 ← Root task router
│
├── _meta/                     ← Layer 0: shared infrastructure
│   ├── ur-prompt.md           ← Domain-agnostic prompt generator
│   ├── runner.md              ← Session protocol
│   ├── prd-intake.md          ← Scaffold a project from a PRD
│   ├── spec-review.md         ← 4-check quality gate (scaffold → specced)
│   ├── status-transitions.md  ← Gate criteria for every status change
│   ├── gap-detection-agent.md ← Converts inference logs → gap objects
│   ├── gap-schema.json        ← Schema for gap objects
│   └── gaps/                  ← Live gap registry
│       ├── pending.txt        ← Inference log (cross-project)
│       └── CONTEXT.md         ← Ranked open gaps
│
├── _intake/                   ← PRD queue
│   └── queue/                 ← Drop PRDs here
│
├── project-alpha/             ← Example project (replace with yours)
│   ├── MANIFEST.md
│   ├── CLAUDE.md
│   ├── CONTEXT.md
│   ├── _planning/             ← ADRs, system design, roadmap
│   ├── _meta/gaps/            ← Project-internal gap registry
│   ├── shared/contracts/      ← Inter-program data shapes
│   └── programs/              ← Runnable components
│       ├── api/
│       └── frontend/
│
└── project-beta/              ← Second example project
    └── programs/cli/
```

---

## Key Rules

1. **Fix-first.** When an agent finds an error, broken reference, or stale content — fix it.
   Do not ask. Identifying and fixing are the same task.
2. **Contracts before code.** A program never builds against a stub contract.
3. **Assumptions block building.** An `assumption` ADR is a hard stop until a human validates it.
4. **spec-review is the only path to specced.** No skipping, no exceptions.
5. **Status lives in MANIFEST.md.** Not in a separate tracker, not in a spreadsheet.
6. **Log every inference.** If you derived something not stated in any file — log it to pending.txt before using it.
