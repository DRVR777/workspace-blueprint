# PRD: Adaptive Living Workspace System (ALWS)
## A Meta-Prompting Recursive Folder Architecture

**Version:** 2.0  
**Status:** Synthesis Document — Design Phase Complete  
**Derived From:** ICM/MWP (Jake's system), ALWS development sessions, meta-prompting pipeline sessions, Dreamworld architecture sessions  
**Core Principle:** "If you don't have at least 3 layers of abstraction, you are a terrorist." — Terry Davis

---

## 1. Executive Summary

ALWS is a self-extending, self-improving, knowledge-accumulating workflow system built entirely from markdown files and folder conventions. It replaces multi-agent frameworks, vector databases, and custom orchestration infrastructure with a recursive folder architecture where **every folder describes itself**, **every agent knows only its layer**, and **the system generates its own next tasks**.

The system has five architectural layers. Each layer knows only the layer directly below it. No layer reaches two layers down. This constraint is non-negotiable and enforced structurally, not by convention.

The two prior systems this synthesizes:

**Jake's ICM/MWP system** solved context discipline for specific, defined content workflows. Folder structure as orchestration. One agent reads the right files at the right moment. Battle-tested, human-operable, glass-box by default. Its limitation: it does not learn, does not detect gaps, does not extend itself.

**The meta-prompting pipeline** solved deterministic lossless knowledge extraction with a folder-based state machine where agents install their own prompts into folders (metaA through metaG). Its limitation: the meta-prompts are domain-coupled; the orchestration is rigid.

ALWS merges the substrate of ICM (folder structure as architecture) with the self-modification logic of the meta-prompting pipeline, adds a universal Ur-Prompt at Layer 0, a fractal MANIFEST protocol at every folder depth, a composition-based knowledge graph distributed across file headers, and an autonomous runner that closes the loop without human intervention except at defined safety gates.

---

## 2. The Terry Davis Constraint

Every architectural decision in this document is evaluated against one test:

> **Could this layer function identically for Byzantine tax law, protein folding, and jazz harmony — without modification?**

If yes, the abstraction is real. If no, domain knowledge has leaked upward and an abstraction boundary has been violated.

The five layers of ALWS satisfy this constraint in sequence:

| Layer | Name | Domain Knowledge | What It Knows |
|-------|------|-----------------|---------------|
| 0 | Ur-Prompt | Zero | The structure of understanding itself |
| 1 | Meta-Registry | Zero | How to route and scaffold workspaces |
| 2 | Workspace Orchestrator | Minimal (task types only) | How to route tasks to pipelines |
| 3 | Stage Pipeline | Specific (domain concepts) | How to execute specific work |
| 4 | Artifact Surface | Specific (output content) | What was produced |

Violations: when Layer 1 material appears in a Layer 3 file, when Layer 0 text references "DevRel" or "essays" or any domain noun, when Layer 2 routing logic reaches into handler implementation details. These are the exact failure modes Claude Code's audit detected in Jake's system — skills-system.md containing verbatim system prompt text (Layer 0 material stored as Layer 3), CONTEXT.md reaching into skill internals.

---

## 3. System Architecture Overview

```
/root/
  CLAUDE.md                        ← Layer 0 orientation (depth 1 only, hard limit)
  CONTEXT.md                       ← Layer 1 routing table
  MANIFEST.md                      ← Root self-description (fractal protocol)
  _meta/                           ← System runtime
    ur-prompt.md                   ← THE UR-PROMPT (Layer 0 generator)
    runner.md                      ← Autonomous execution loop
    MANIFEST.md
    gaps/
      CONTEXT.md                   ← Gap registry (priority table)
      pending.txt                  ← Inference log
      gap-NNN-[description].json   ← Individual gap objects
    improvement-engine/            ← Self-modification pipeline
    graph-engine/                  ← Knowledge graph materialization
      graph.json                   ← Materialized graph (cache)
      graph-builder.md             ← Agent: rebuild graph from headers
      query-engine.md              ← Agent: traverse graph.json
  _registry/                       ← Task type registry
    REGISTRY.md                    ← Master map: task-type → workspace
    type-signatures/               ← Structural fingerprints per type
    workspace-builder/             ← Scaffolds new workspaces
    fallback/                      ← Unknown task type handler
  [workspace-A]/                   ← Registered task-type workspace
    MANIFEST.md
    CONTEXT.md
    stages/
      01-[name]/
        MANIFEST.md
        CONTEXT.md                 ← Stage contract (inputs/process/outputs)
        handlers/                  ← Meta-prompts per document type
        output/
        diff-log/
      02-[name]/
        ...
  [workspace-B]/
    ...
  _intake/
    MANIFEST.md
    queue/                         ← Raw inputs land here
    processing/                    ← Active pipeline runs
    processed/                     ← Archived with pointer to final location
    needs-review/                  ← Below-confidence-threshold items
  _knowledge/
    MANIFEST.md
    graph.json                     ← Materialized knowledge graph (cache)
    [domain]/
      [subdomain]/
        [concept-id].md            ← Composition-based concept files
```

---

## 4. Layer 0: The Ur-Prompt

### What It Is

The Ur-Prompt is the only file in the system that is **never modified by the system itself**. It is written once. It knows nothing about any domain. It takes a gap object as input and produces a 5-part execution prompt as output.

It knows four things and only four things:

**1. The universal structure of a concept** (domain-independent, derived from epistemology):
- Every concept is composed of other concepts
- Those concepts were combined in a way that was non-obvious before the combination existed
- That combination produces properties none of the parts have alone
- The concept is itself a component of larger concepts
- The concept has boundaries — places where it ends and adjacent concepts begin

This is true of a plane. A legal argument. A musical chord. A protein. A political system. A mathematical proof. The structure is universal. The content is irrelevant to Layer 0.

**2. The three gap types** (domain-independent, derived from graph theory):
- **Type 1 — Missing Composition**: A node exists but its component structure is unknown. The gap is downward: what is this made of?
- **Type 2 — Missing Bridge**: Two nodes exist but no edge connects them. The gap is lateral: how do these relate?
- **Type 3 — Shallow Node**: A node exists and has a composition table, but its components aren't themselves expanded. The gap is recursive: how deep does this go?

Every possible gap in any knowledge graph is exactly one of these three. Nothing else.

**3. The five properties of a good execution prompt**:
- **Position**: Shows the agent exactly where in the structure the gap is. The agent sees the surrounding nodes.
- **Shape**: Defines the exact output format the agent must produce. Not what to say — what shape the saying must have.
- **Constraint**: Removes degrees of freedom that would allow the agent to produce something that doesn't fit the structure.
- **Scope**: Tells the agent exactly how far to go. Fill this gap. Not adjacent gaps. Not the whole domain.
- **Verification**: Embeds a self-check the agent runs before submitting. "Does your output fill the gap described in Position? If no, do not submit."

A prompt missing any of these five properties is incomplete.

**4. The self-test**:
> "Would this generated prompt work equally well for Byzantine tax law?"
> If yes, Layer 0 is clean. If no, domain knowledge has leaked upward.

### The Ur-Prompt File

```markdown
---
id: ur-prompt-v1
type: layer-0-generator
domain: null
version: 1.0.0
immutable: true
---

# The Ur-Prompt

You are not an assistant. You are a prompt architect.

You will be given a gap object describing a missing piece in a structure of knowledge.
You know nothing about the domain of that knowledge.
You do not need to.

Your only job is to understand the SHAPE of the gap and generate
a prompt that will cause an agent to fill that shape precisely.

## What You Know About Knowledge (Domain-Independent)

Every concept in any domain of knowledge has the same structure:
1. It is composed of other concepts
2. Those concepts were combined in a way that was non-obvious
3. That combination produces properties none of the parts have alone
4. It is itself a component of larger concepts
5. It has boundaries — places where it ends and adjacent concepts begin

## What You Know About Gaps (Domain-Independent)

A gap is one of three things, always:
1. A node exists but its composition is unknown → gap is DOWNWARD
2. Two nodes exist but no edge connects them → gap is LATERAL
3. A node exists but its components aren't expanded → gap is RECURSIVE

## What You Know About Good Prompts (Domain-Independent)

Every generated prompt must have all five:
POSITION / SHAPE / CONSTRAINT / SCOPE / VERIFICATION

## Input Format

{
  "gap_type": "missing_composition" | "missing_bridge" | "shallow_node",
  "target": "[concept identifier]",
  "context": "[what the graph currently knows near this gap]",
  "priority": "[why this gap matters to the structure]",
  "depth": "[how deep in the understanding tree this sits]"
}

## Output Format

---
POSITION:
[Show the agent where it is. What exists. What is adjacent. What is missing.
 No domain knowledge from you. Derive from context only.]

SHAPE:
[Define the exact output format. Tables, edges, declarations.
 The shape must be rejecting: bad outputs must not fit it.]

CONSTRAINT:
[What the agent must NOT do. Always include:
 - Do not summarize
 - Do not explain history
 - Do not fill adjacent gaps
 - Do not produce prose where structure is required]

SCOPE:
[Define exactly where this prompt ends.
 What counts as done. What counts as out of scope.]

VERIFICATION:
[The self-check the agent runs before submitting.
 Must be binary: gap filled or not filled.]
---

## The One Rule

If you find yourself knowing anything about what the answer
should be — stop. You have collapsed into Layer 2.

Your output is equally correct for nuclear deterrence, protein folding,
jazz harmony, or Byzantine tax law.

That is the test. Apply it before submitting.
```

### Ur-Prompt Execution Protocol (Steps 1-7)

When filling a gap, an agent executing the Ur-Prompt follows these steps explicitly:

1. **Receive** the gap object (JSON)
2. **Classify** the gap type (missing_composition / missing_bridge / shallow_node)
3. **Extract** the surrounding graph context from the gap's `context` field
4. **Generate** POSITION — derive from context, zero domain inference
5. **Generate** SHAPE — match to gap type (composition = table; bridge = edge declaration; shallow = recursive table)
6. **Generate** CONSTRAINT + SCOPE + VERIFICATION
7. **Apply the self-test** — "Would this work for Byzantine tax law?" If no, revise.

---

## 5. Layer 1: The Meta-Registry

### What It Is

The registry is the system that builds systems. It knows what workspaces exist and generates new ones when needed. It does not execute work — it routes and scaffolds.

### REGISTRY.md Structure

```markdown
| Task Type            | Workspace Path                  | Version | Runs | Avg Quality | Status |
|---------------------|---------------------------------|---------|------|-------------|--------|
| knowledge-extraction | /_registry/knowledge-extraction/ | v3      | 47   | 0.94        | active |
| essay-writing        | /_registry/essay-writing/        | v2      | 12   | 0.87        | active |
| architecture-planning| /_registry/architecture-planning/| v1      | 3    | 0.81        | active |
| code-scaffolding     | /_registry/code-scaffolding/     | v1      | 8    | 0.89        | active |
```

`Avg Quality` is populated by the validation layer after each run. `Version` increments when the workspace self-modifies. These are the only two fields that change automatically. All other fields require human authorization to modify.

### Type Detection

When a task arrives, the registry runs type detection before routing:

```json
{
  "primary_type": "knowledge-extraction",
  "confidence": 0.87,
  "alternatives": [
    { "type": "research-task", "confidence": 0.61 }
  ],
  "detected_facets": ["has_document_input", "has_concept_output", "structured_source"],
  "flags": []
}
```

If confidence is below 0.75, the task routes to `_registry/fallback/` — a discovery workspace that runs a 5-stage pipeline to understand the new task type, generate its fingerprint, scaffold a workspace for it, and register it. The first run of a new workspace type will produce lower quality than mature workspaces. That is expected. The Avg Quality field tracks this.

### The Workspace Builder

`_registry/workspace-builder/` is itself an ICM workspace with stages:

```
01-discovery/       ← understand the new task type from examples
02-fingerprint/     ← create type signature for future detection
03-scaffold/        ← generate workspace folder structure
04-handler-gen/     ← generate initial meta-prompts per stage
05-register/        ← add to REGISTRY.md
```

The workspace builder can be invoked directly for PRD intake (see Section 11).

---

## 6. The Fractal MANIFEST Protocol

### Core Principle

Every folder in the system — at every depth, without exception — contains a MANIFEST.md. The schema is identical at every level. An agent dropped anywhere in the tree can reconstruct full context by reading three documents: its current MANIFEST, the parent pointer chain to root, and the child MANIFEST of whichever subfolder it's routing into.

This is the TCP/IP model applied to filesystem navigation:

| TCP/IP Layer | Folder System Layer |
|---|---|
| Application | Task content (the actual work) |
| Transport | Stage pipeline (sequencing, reliability) |
| Network | Workspace routing (where does this go) |
| Data Link | MANIFEST.md (how to parse this folder) |
| Physical | The filesystem itself |

The MANIFEST is the data link layer. It frames the contents of the folder in a way that the network layer (routing) can handle without understanding the application layer (the actual task).

### MANIFEST.md Schema

```markdown
---
id: [unique identifier]
type: workspace | stage | handler | artifact | config | registry | meta
parent: [path to parent folder]
depth: [integer from root, 0 = root]
created: [ISO8601 timestamp]
version: [semver]
status: active | archived | building | failed | reconciled
---

## What I Am
[One sentence. No jargon. An agent that has never seen this system
 should understand this folder's purpose from this line alone.]

## What I Contain
| Name          | Type   | Status   | Purpose                              |
|---------------|--------|----------|--------------------------------------|
| stages/       | folder | active   | Sequential processing pipeline       |
| CONTEXT.md    | file   | active   | Routing instructions for this level  |
| output/       | folder | active   | Stage artifacts                      |

## What I Need From Parent
[What this folder expects to receive as input context]

## What I Give To Children
[What context this folder injects when a child is processing]

## What I Return To Parent
[What this folder's output means to the layer above it]

## Routing Rules
| If task involves...   | Go to...           |
|----------------------|--------------------|
| [condition]          | [subfolder/file]   |
```

### MANIFEST Maintenance Rules

Three mechanisms keep MANIFESTs accurate without manual intervention:

**1. Write-through on creation**: Any agent that creates a folder creates its MANIFEST first, before writing anything else. Folder creation is a two-step atomic operation: (1) create folder + write MANIFEST with `status: building`, (2) write contents. If step 2 fails, the MANIFEST correctly shows `status: building` — interrupted, not corrupted.

**2. Write-through on modification**: You cannot write to a folder without updating its MANIFEST's `What I Contain` table. This is not a separate step — it is part of every write operation.

**3. Periodic reconciliation**: A reconciliation agent runs after each pipeline stage. It walks the tree, compares declared contents to actual filesystem contents, flags discrepancies in `diff-log/`. A folder without a MANIFEST gets an auto-generated one marked `status: reconciled` (not active — signals auto-generated, may be incomplete).

### The Metadata Gradient

The MANIFEST at each level describes only its own level but contains pointers upward and downward. This makes the fractal navigable without loading the entire tree:

```
Root MANIFEST
  ↓ knows about workspace-level folders
  ↓ does NOT know about stage-level folders

Workspace MANIFEST
  ↑ pointer to root
  ↓ knows about stage-level folders
  ↓ does NOT know about handler-level folders

Stage MANIFEST
  ↑ pointer to workspace
  ↓ knows about handler-level folders
  ↓ does NOT know about artifact-level folders
```

An agent navigates the tree by reading one MANIFEST at a time, deciding up or down, reading the next MANIFEST. Three reads. Full orientation. At any depth.

---

## 7. Layer 2: The Workspace Orchestrator

### CLAUDE.md — The Hard Constraint

CLAUDE.md maps **depth 1 only**. This is a hard constraint, not a guideline. If CLAUDE.md contains folder details below the workspace level, it burns context budget on irrelevant structure every time an agent is loaded.

**Correct CLAUDE.md structure:**

```markdown
# System Orientation

You are working inside an Adaptive Living Workspace System.
Read _meta/ur-prompt.md for the Layer 0 architecture.
Read CONTEXT.md for routing to specific workspaces.

## Workspaces Available
| Workspace             | Purpose (one line)                       |
|----------------------|------------------------------------------|
| knowledge-extraction/ | Process documents into structured knowledge |
| architecture-planning/| PRD to executable plan pipeline          |
| essay-writing/        | Research to polished long-form output    |
| _intake/              | Route raw inputs to the right workspace  |
| _meta/                | System runtime, gaps, improvement engine |

<!-- See _meta/ur-prompt.md for Layer 0 architecture.
     Each workspace's CONTEXT.md maps its own internals. -->
```

That is the entirety of CLAUDE.md. No handler implementations. No stage details. No skill internals. Workspace names and one-line purposes only.

### Root CONTEXT.md — The Routing Table

The routing table is the most important single pattern in the system. It does not describe work. It routes work to wherever work is described.

```markdown
# Root Routing Table

## Task Intake
| If task contains...           | Route to...                    |
|------------------------------|--------------------------------|
| A raw document or text input  | _intake/                       |
| A PRD document                | _registry/workspace-builder/   |
| An explicit workspace name    | That workspace's CONTEXT.md    |
| An unknown task type          | _registry/fallback/            |

## Active Workspaces
| Workspace             | Entry Point              |
|----------------------|--------------------------|
| knowledge-extraction/ | stages/01-ingress/       |
| architecture-planning/| stages/01-discovery/     |
| essay-writing/        | stages/01-research/      |

## System Operations
| Operation                | Route to...              |
|--------------------------|--------------------------|
| Process pending gaps     | _meta/runner.md          |
| Add to knowledge graph   | _meta/graph-engine/      |
| Self-modification review | _meta/improvement-engine/|
```

---

## 8. Layer 3: The Stage Pipeline

### Stage Contract Structure

Every stage has a CONTEXT.md with this exact structure. No stage contract is optional:

```markdown
# Stage [N]: [Name]

## Inputs
| Source         | File              | Facets Required                    | Why                        |
|----------------|-------------------|------------------------------------|----------------------------|
| Previous stage | ../[N-1]/output/  | [specific sections to load]        | [reason for each facet]    |
| Config         | ../../_config/    | [specific config sections]         | [reason]                   |

## Process
[What the agent does. Described as operations on inputs to produce outputs.
 Not a description of domain knowledge. Not a summary of what the output says.
 What computational steps transform inputs into outputs.]

## Checkpoint
[What the agent presents to the human before committing output.
 Options must be MEANINGFULLY distinct — not just numbered variations.
 The checkpoint only has value if a human can make a real decision here.]

## Audit (run before generating output)
[Checklist the agent runs against its own planned output.
 Binary checks only: present/absent, not subjective quality assessments.]
- [ ] [Check 1]
- [ ] [Check 2]

## Outputs
| File              | Format  | Contents                           |
|-------------------|---------|-------------------------------------|
| output/[file.ext] | [format]| [what this file must contain]       |

## Verification Gate
[The condition that must be true for this stage to be considered complete.
 If this condition is not met, the stage has not finished and
 the next stage must not start.]
```

### Faceted Context Loading

The key improvement over vanilla ICM: context loading is specified at the **facet level**, not the file level. A "voice rules" document might have sections for hard constraints, strategic rationale, and historical examples. The stage contract specifies which sections to load.

This matters because token budget is finite. Loading a 3000-word voice document when you need 200 words of hard constraints is waste. The facet system enforces the context discipline ICM promises but doesn't always deliver.

### Handler Registry Pattern

Meta-prompts are handlers in the registry sense. They are registered by document type. The extractor handler for RAW_PDF is different from the one for NOTES. When a new document type appears, you register a new handler. No existing handlers change.

```
/stages/02-extract/
  CONTEXT.md              ← stage contract
  handlers/
    RAW_PDF.md            ← extraction rules for OCR output
    NOTES.md              ← extraction rules for class notes
    READING.md            ← extraction rules for curated readings
    _fallback.md          ← generic extraction for unknown types
  output/
  diff-log/
```

The CONTEXT.md `Inputs` table specifies which handler to load based on the `document_type` field from the previous stage's output. The document type is the routing key. The handler is the route destination.

### The Diff Log

Every stage has a `diff-log/` subfolder. When an agent modifies a stage contract based on what it learned during execution, it writes to `diff-log/YYYY-MM-DD.md`:

```markdown
## Change: Added noise_ratio threshold check to Inputs
**Trigger**: Encountered RAW_PDF with 40% noise ratio that corrupted extraction
**Change**: Added precondition — if noise_ratio > 0.35, run clean stage before extract
**Effect**: Downstream extraction quality on dense PDFs improved significantly
**Confidence**: High — pattern appeared in 3 consecutive runs
**Runs Until Graduation**: 0 (threshold met, ready for CONTEXT.md modification)
```

A change graduates from diff log to actual CONTEXT.md modification when: it appears in the diff log across ≥3 runs, quality metrics improve after the change, and (for multi-stage changes) a human has approved it.

---

## 9. The Knowledge Graph System

### Folder Structure ≠ Graph

A folder structure is a tree — exactly one path between any two nodes, no cycles, no cross-links. A knowledge graph is a directed graph — any node can point to any other node, cycles are legal, relationships have types.

ALWS uses both. The folder tree is the storage layer. The knowledge graph is the relationship layer. The graph is distributed across file headers and materialized into `graph.json` by the graph-builder agent.

### The Composition-Based Knowledge Model

The critical insight that renders "one concept per file" wrong: **a concept is never self-contained**. It is always a specific composition of other concepts that have never been combined that way before. The plane isn't metal + engine + wings listed separately — it is the emergent property of how those things combine. The concept lives in the relationship, not in the parts.

Every concept file has three sections:

```markdown
---
id: [concept-id]
type: knowledge-atom
path: /[domain]/[subdomain]/[concept-id].md
edges:
  - target: [component-concept-id]
    type: requires | instantiates | enables | contradicts | refines
    weight: [0.0-1.0]
created: [ISO8601]
version: [semver]
---

## What This Is
[One sentence definition. Not a paragraph. One sentence.]

## What This Is Made Of
| Component       | Concept ID         | Role in This Concept                |
|-----------------|--------------------|-------------------------------------|
| [component]     | [concept-id]       | [how this component contributes]    |

## What Makes This Composition Novel
[The non-obvious insight. What is true of the composition that
 is NOT true of any individual component.
 This is the intellectually irreducible part of this concept.]

## What This Is A Component Of
| Parent Concept  | Concept ID         | How This Contributes                |
|-----------------|--------------------|-------------------------------------|
| [parent]        | [concept-id]       | [role this concept plays in parent] |
```

The "What This Is Made Of" table is the **mandatory validation gate**. If an extractor cannot fill this table, the concept has not been understood — it has been named. The diff log should flag any concept file with an empty composition table as a `missing_composition` gap.

### The Materialized Graph

`_meta/graph-engine/graph.json` is the queryable version of the knowledge graph — not the authoritative version (that lives in file headers) but the navigable cache:

```json
{
  "nodes": {
    "[concept-id]": {
      "path": "/[domain]/[subdomain]/[concept-id].md",
      "type": "knowledge-atom",
      "tags": ["[tag1]", "[tag2]"]
    }
  },
  "edges": [
    {
      "from": "[concept-id-a]",
      "to": "[concept-id-b]",
      "type": "requires",
      "weight": 0.95
    }
  ]
}
```

The graph-builder agent regenerates `graph.json` by walking the folder tree and reading every file's `edges` field. It runs: after intake pipeline places a new file, after the improvement engine modifies a file, and on a schedule as a consistency check.

### What the Graph Enables That Folders Cannot

Once `graph.json` exists, the system can answer questions impossible with folder navigation:

**Traversal queries**: "Give me everything within 2 hops of [concept]" — crosses folder boundaries.

**Gap detection**: "Which nodes have no incoming edges?" — orphan concepts, candidates for deletion or connection.

**Path finding**: "What is the conceptual path from [concept-A] to [concept-B]?" — traces prerequisite chains across the whole tree.

**Contradiction surfacing**: "Are there cycles of type contradicts→contradicts?" — surfaces real tensions worth examining.

**Subgraph extraction**: "Give me all nodes tagged [domain] and their interconnections" — the foundation for synthesis and category decomposition.

---

## 10. The Gap Detection Engine

### The Three Gap Types as JSON

```json
// Type 1: Missing Composition
{
  "gap_type": "missing_composition",
  "concept": "[concept-id]",
  "existing_edges": [],
  "expected_depth": 3,
  "actual_depth": 0,
  "priority": "high",
  "requires_human": false
}

// Type 2: Missing Bridge
{
  "gap_type": "missing_bridge",
  "concept_a": "[concept-id-a]",
  "concept_b": "[concept-id-b]",
  "shared_subcomponents": ["[shared-concept-id]"],
  "bridge_hypothesis": "[one sentence describing the likely relationship]",
  "priority": "medium",
  "requires_human": false
}

// Type 3: Shallow Node
{
  "gap_type": "shallow_node",
  "concept": "[concept-id]",
  "composition_declared": ["[component-1]", "[component-2]"],
  "components_in_graph": 0,
  "priority": "low",
  "requires_human": false
}
```

### Priority Formula

```
Priority = (concept_in_degree × 0.4) + (concept_depth_deficit × 0.3) + (bridge_shared_components × 0.3)
```

Concepts that many other concepts depend on get filled first. This is the topological sort of the understanding tree applied to expansion order — foundations before superstructures.

### The Gap Registry (CONTEXT.md in _meta/gaps/)

```markdown
# Gap Registry

| Gap ID  | Type                  | Target              | Severity  | Status | Requires Human |
|---------|----------------------|---------------------|-----------|--------|----------------|
| gap-001 | missing_composition   | concept-deterrence  | blocking  | open   | false          |
| gap-002 | missing_bridge        | rational-actor ↔ MAD| degrading | open   | false          |
| gap-003 | shallow_node          | concept-jet-engine  | cosmetic  | closed | false          |
| gap-004 | missing_composition   | CLAUDE.md Layer 0   | blocking  | open   | true           |
```

Severity tiers: **blocking** (other work cannot proceed), **degrading** (quality is impaired), **cosmetic** (minor completeness issue).

`requires_human: true` for any gap that: modifies CLAUDE.md or root CONTEXT.md, adds a new workspace folder, touches the Ur-Prompt itself, or crosses more than two stage boundaries. This is the Terry Davis constraint applied to autonomy — the system runs freely within its layer, stops at layer boundaries, requests permission to cross.

### The Inference Log (pending.txt)

```
[2026-03-13T14:22:00Z] | stage-extract | RAW_PDF.md | inferred "footnotes contain substantive content not noise" — no file states this
[2026-03-13T16:45:00Z] | stage-validate | confidence.json | inferred threshold 0.91 means "good" — no calibration file defines quality thresholds
[2026-03-14T09:30:00Z] | ur-prompt | gap-005 | derived 6 content frame types from synthesis of voice.md + domain knowledge — not stated in any source file
```

Every inference made by every agent, logged before the inference is committed to any permanent file. The inference log is what makes the system's epistemic state auditable. A human reviewing any output can trace every assumption to either a source file or a pending.txt entry.

---

## 11. The Intake Pipeline

### End-to-End Flow

```
raw text input
      ↓
01-envelope      ← wrap in packet format, assign ID, hash, timestamp
      ↓
02-classify      ← detect type, produce verdict with confidence
      ↓
03-route         ← find location in folder tree
      ↓
04-place         ← write file with full metadata header
      ↓
05-update        ← update every MANIFEST on path to root + update graph.json
```

### The Envelope Stage

The first operation for any input: wrap it in a standard packet. No classification yet. Just give it an ID and preserve the original.

```markdown
---
id: intake-2026-03-13-0047
status: unprocessed
received: 2026-03-13T14:23:11Z
raw_hash: sha256:[hash]
source: manual_input | prd-intake | api | scheduled
---

[raw text, untouched]
```

This file lands in `_intake/queue/`. Even if every subsequent stage fails, the original input exists exactly as received.

### Classification

The classifier looks for structural signals, not domain knowledge:

| Signal Type | Examples | Verdict |
|---|---|---|
| Structural markers | `# Introduction`, numbered lists | document |
| Temporal language | "yesterday", "on March 3" | note/event |
| Question form | "how does X work?" | research-task |
| Imperative + steps | "first do X, then Y" | procedure |
| Concept definition | "X is a Y that Z" | knowledge-atom |
| URL or reference | `https://...`, "see Smith 2019" | reference |
| Code blocks | ``` blocks, function signatures | code-artifact |
| Named entities | person + date + event | record |

Below 0.75 confidence → route to `_intake/needs-review/`. The human decides. The decision is logged as a high-confidence training example that improves future classification.

### Routing

Routing is MANIFEST-driven and local at every hop:

1. Type verdict maps to a top-level workspace
2. Agent reads that workspace's MANIFEST and uses routing rules to navigate down the tree
3. At each level: read MANIFEST, make one routing decision, move one level deeper
4. Before placement: similarity check against items in the target folder

```json
{
  "target_path": "/[domain]/[subdomain]/[category]/",
  "routing_confidence": 0.91,
  "routing_path": [
    { "folder": "/[domain]/", "rule_applied": "type=[type]" },
    { "folder": "/[domain]/[subdomain]/", "rule_applied": "tag=[tag]" }
  ],
  "duplicate_risk": 0.23,
  "potential_duplicate": null
}
```

### Placement

Five things happen simultaneously on placement:

1. File is written to final location with full metadata header
2. Every MANIFEST on the path to root is updated
3. Knowledge graph is updated: new node added, related items get back-edges
4. Intake queue entry archived to `_intake/processed/` with pointer to final location
5. Routing decision logged with confidence score in `pending.txt` for improvement tracking

---

## 12. The PRD Intake Pipeline

### What It Does

Given a PRD document, the system: extracts structured project information, checks for naming collisions, scaffolds a complete workspace with stages, generates handler meta-prompts, registers the new workspace type, and flags all inferences made during scaffolding as assumption ADRs requiring human validation.

### Extraction Phase (Before Any File Creation)

Before touching the filesystem, the agent extracts:

1. **Project name** (explicit or derived)
2. **One-liner** (what this project does, ≤15 words)
3. **Programs/components** (explicit or inferred from architecture description)
4. **Shared contracts** (APIs, data schemas, interfaces between components)
5. **Decisions already made** (explicit statements in the PRD that constrain implementation)
6. **Unknowns** (what the PRD doesn't answer — these become blocking gaps)

Unknowns become `requires_human: true` gap objects filed immediately. Blocking gaps prevent dependent stages from running until resolved.

### ADR Distinction

Two types of Architecture Decision Records (ADRs) are generated:

**Accepted**: Decisions explicitly stated in the PRD. Status: `accepted`. Can be built against immediately.

**Assumption**: Decisions the agent inferred to fill PRD gaps. Status: `assumption`. Must be validated by a human before the component they affect is built.

This is the distinction that prevents "clean-looking scaffolding built on wrong assumptions" — the most common failure mode when auto-generating from PRDs.

### Scaffolding Order

```
Step 1: Root MANIFEST (project is identifiable before anything else exists)
Step 2: Planning workspace + stub ADRs
Step 3: Contract definitions (programs can't reference each other without interfaces)
Step 4: Individual program scaffolds (leaf nodes, built after their dependencies)
Step 5: Cross-program dependency registration in root gap registry
Step 6: REGISTRY.md entry for the new project type
```

---

## 13. The Runner (Autonomous Execution Loop)

### runner.md

```markdown
# Runner — Autonomous Gap Resolution

## Your Instruction

Do not ask what to do. Read the gap registry and do the next thing.

## Step 1 — Process Pending Inferences

Read `_meta/gaps/pending.txt`.
If it has unprocessed entries:
  Run gap detection against those entries.
  Write new gap JSON files.
  Update gap registry.
  Mark entries processed.
If empty: skip to Step 2.

## Step 2 — Select Next Gap

Read `_meta/gaps/CONTEXT.md`.
Priority order: blocking → degrading → cosmetic.
Within same severity: lowest gap number first (oldest).
If no open gaps: report "Registry clear." and stop.
If highest-priority gap has requires_human: true:
  Report: "Gap-[N] requires human review. Reason: [field]. 
  Set requires_human to false to proceed."
  Stop.

## Step 3 — Generate Execution Prompt

Feed selected gap object to `_meta/ur-prompt.md` Steps 1–7.
Produce full 5-part execution prompt.
State: "Executing gap-[N]: [one line description]"

## Step 4 — Execute

Run the execution prompt.
Follow CONSTRAINT and SCOPE exactly.
Run VERIFICATION before submitting output.

## Step 5 — Close the Loop

Update gap JSON: status → closed.
Update gap registry.
Append inferences from execution to pending.txt immediately
  (not at end — as they occur).
State: "Gap-[N] closed. Next: [gap-N+1 description or 'registry clear']."

## One Rule

If you make an inference not supported by a loaded file:
append it to pending.txt BEFORE continuing.
Do not wait until the end.
```

### Invoking the Runner

For a single pass: `"Read _meta/runner.md and execute."`

For continuous operation: `"Read _meta/runner.md and execute until registry clear or human gate hit."`

For automated scheduling (shell):

```bash
#!/bin/bash
cd /path/to/workspace
claude --print "Read _meta/runner.md and execute until registry clear or human gate hit." \
       --output-format text >> _meta/gaps/run-log.txt
```

---

## 14. The Improvement Engine

### What It Is

A self-contained ICM workspace that runs after sessions to improve handler quality. It reads telemetry from completed runs, detects patterns, proposes handler modifications, validates them against historical data, and applies approved changes.

### Stage Structure

```
_meta/improvement-engine/
  MANIFEST.md
  CONTEXT.md
  stages/
    01-aggregate/       ← collect telemetry.json from last N runs
    02-pattern-detect/  ← find recurring failures
    03-propose/         ← generate specific handler modifications as diffs
    04-validate/        ← test proposals against historical runs
    05-commit/          ← apply approved modifications
```

### Quality Telemetry

Every stage writes a `telemetry.json` alongside its output:

```json
{
  "stage": "02-extract",
  "run_id": "2026-03-13-001",
  "document_type": "RAW_PDF",
  "handler_used": "handlers/RAW_PDF.md",
  "metrics": {
    "concept_coverage": 0.94,
    "composition_table_completeness": 0.87,
    "noise_removed": 0.23
  },
  "issues_flagged": 2,
  "human_edits_made": true,
  "edit_summary": "Removed 3 false concepts from dense footnote section"
}
```

Human edits are the highest-signal feedback in the system. When a human opens an output and changes it, that is evidence about where the handler failed. `human_edits_made: true` is the trigger for the improvement engine to prioritize that stage in its next run.

### Confidence-Gated Self-Modification

Stage contracts are immutable within a single run. Between runs, they become mutable only when:

1. The proposed change appears in the diff log across ≥3 runs
2. Quality metrics improve after the change (validated against historical telemetry)
3. For single-stage, no-downstream-effect changes: applies automatically, logs the change
4. For multi-stage, schema-changing, or cross-workspace changes: writes to human review queue, waits

This prevents circular reasoning — the system cannot restructure toward local optima without accumulated evidence.

### Proposal Format

Proposals are written as diffs, not replacements:

```markdown
## Proposed Change: gap-detection/RAW_PDF handler
**Type**: precondition addition (single-stage, low-risk)
**Evidence**: 3 runs (gap-001, gap-004, gap-009) — noise_ratio > 0.30 correlates with concept_coverage < 0.80
**Proposed Addition**: Add to stage CONTEXT.md Inputs:
  "If noise_ratio > 0.30: load handlers/pre-clean.md before handlers/RAW_PDF.md"
**Expected Impact**: +0.08 concept_coverage on dense PDFs
**Auto-apply eligible**: true (single-stage, quality-improving, ≥3 runs of evidence)
```

---

## 15. Failure Modes and Architectural Guards

### Guard 1: Circular Self-Modification

**Failure mode**: Agent modifies its own routing instructions to optimize for the current task rather than the system's long-term quality.

**Guard**: No stage can modify its own CONTEXT.md during a run. Self-modification only happens through the improvement engine between runs, with confidence thresholds and human review on high-impact changes. An agent editing its own routing during execution is a protocol violation, not a feature.

### Guard 2: Quality Regression

**Failure mode**: A proposed modification improves performance on recent runs but degrades performance on earlier task types.

**Guard**: Every modification is applied to a shadow copy and validated against historical telemetry before touching the live workspace. If quality metrics decline on any task type, the modification is rejected and logged.

### Guard 3: Cascade Failure

**Failure mode**: An unvalidated output propagates through the pipeline, causing failures five stages later that are hard to trace back to their source.

**Guard**: The `finalized.flag` system. A stage that fails validation cannot write `finalized.flag`. The next stage's CONTEXT.md includes the check: "If ../[previous-stage]/finalized.flag does not exist: halt and report." Unfinalized stages do not poison synthesis.

### Guard 4: Abstraction Collapse

**Failure mode**: A component starts reaching two layers down, coupling layers that should be isolated.

**Guard**: When a component needs information two layers away, that is the signal a new intermediate layer is needed — not that the boundary should be violated. The self-test from the Ur-Prompt ("would this work for Byzantine tax law?") is the diagnostic for Layer 0 collapse. For lower layers: "would this CONTEXT.md make sense in a different workspace?" is the diagnostic.

### Guard 5: MANIFEST Staleness

**Failure mode**: A human manually adds a file, no MANIFEST updates, downstream agents navigate stale structure.

**Guard**: The reconciliation agent. It detects discrepancies and marks affected MANIFESTs `status: reconciled`. Agents reading a `reconciled` MANIFEST are on notice that it may be incomplete. The gap detection engine treats `reconciled` MANIFESTs as a `missing_composition` gap — something in that folder has not been properly described.

### Guard 6: Footnote/Noise Boundary Ambiguity

**Failure mode**: Noise stripping in RAW_PDF documents removes substantive footnotes (which contain critical citations and qualifications) along with genuine noise (page numbers, image captions, headers).

**Guard**: The noise stripping handler must maintain a `noise-decisions.md` file listing every stripped segment and the rule that triggered its removal. When concept_coverage drops below threshold, the improvement engine checks noise-decisions.md first. Any stripped segment that appears as a referenced concept in other extracted files is a mis-strip — log as `missing_composition` gap, restore segment, re-run extraction.

### Guard 7: PRD Scaffolding on Wrong Assumptions

**Failure mode**: Auto-generated workspace looks complete but is built on inferences that turn out to be wrong.

**Guard**: The ADR distinction. Every inference made during PRD scaffolding is written as an `assumption` ADR. No program that depends on an `assumption` ADR can move past Stage 1 until the assumption is validated or rejected by a human. The gap registry shows all open assumption ADRs as `blocking` with `requires_human: true`.

---

## 16. Implementation Sequence

### Phase 0: Substrate (Week 1)
Build what Jake already built, exactly. Three-layer ICM for one specific domain. CLAUDE.md at root with depth-1 map. CONTEXT.md routing table. One workspace with numbered stages and stage contracts. Run it against real tasks until it produces consistent outputs. Do not add any ALWS features until Phase 0 produces reliable output.

Phase 0 is the proof that the substrate works. ALWS built on a broken substrate will fail for reasons that are hard to distinguish from ALWS architectural flaws.

### Phase 1: Fractal Protocol (Week 2)
Add MANIFEST.md to every folder. Start with the existing workspace folders — add MANIFEST to each stage folder. Test the fractal test: drop an agent into the deepest stage folder with only the MANIFEST available. Can it orient? Can it follow parent pointers to root? Can it navigate to its output folder? If yes, the fractal is real. If no, revise the MANIFEST template.

### Phase 2: Meta Layer (Week 3)
Add `_meta/` with the Ur-Prompt, runner.md, and `gaps/`. Seed the gap registry with 3-5 gaps you already know exist from Phase 1 operation. Run the runner. Watch it process gaps. Verify that pending.txt accumulates inferences. Verify that the Ur-Prompt generates structured 5-part prompts, not prose descriptions.

### Phase 3: Knowledge Graph (Week 4)
Convert the first workspace's outputs to composition-based concept files. Add `edges` fields to their headers. Run graph-builder to generate graph.json. Run one traversal query by hand to verify the graph is navigable. Add the gap detection engine configured to scan concept files for empty composition tables.

### Phase 4: Intake Pipeline (Week 5)
Build the 5-stage intake pipeline. Test with a document you've already processed manually — compare auto-classified location against where you would have filed it. Tune the classifier. Test MANIFEST update propagation: after intake, verify that every parent MANIFEST shows the new file.

### Phase 5: Improvement Engine (Week 6)
Add telemetry.json to all stage outputs. Run the improvement engine against telemetry from Phases 1-4. Verify it produces diff-format proposals, not replacement proposals. Test the shadow-copy validation: apply a proposal to a shadow, run it against historical telemetry, verify the accept/reject logic works.

### Phase 6: PRD Intake + Registry (Week 7-8)
Build the workspace-builder and PRD intake pipeline. Test by feeding a real PRD. Verify: extraction is correct, ADR distinction is made, assumption ADRs block downstream stages, the new workspace is registered in REGISTRY.md and has a valid type fingerprint.

### Phase 7: Autonomous Operation (Week 9)
Wire up the runner to run automatically. Set the schedule. Monitor run-log.txt for the first week. Human review every human-gated gap. After one week of clean autonomous runs, expand the schedule.

---

## 17. What This System Replaces

**Multi-agent frameworks (CrewAI, AutoGen, LangChain)**: For sequential, stage-based workflows, the folder structure is the orchestration layer. One agent reading the right files at the right moment produces the same coordination without the dependency on external frameworks, without the debugging complexity of distributed agent communication, and without the opacity that makes failures hard to trace.

**Vector databases**: The knowledge graph + naming conventions + materialized graph.json replace semantic search for most practical use cases. Agents know where to look because the MANIFEST structure and routing tables tell them. The graph handles relationship traversal. When a file is placed, it is connected to related items by the intake pipeline — the connections are explicit, not inferred at query time.

**Custom Python orchestration infrastructure**: No deployment pipelines, no servers, no compiled code. A folder is an app. It can be versioned with Git, shared as a ZIP, run by anyone with Claude Code. The glass-box property — every stage readable by a non-technical collaborator — is preserved because nothing is ever hidden inside a framework they can't see.

**Manual prompt maintenance**: The improvement engine updates handlers based on accumulated evidence. Prompts stop being artifacts you maintain and become artifacts that maintain themselves, with human oversight on consequential changes. The diff log is the version history of your prompts, with justification for every change.

---

## 18. Success Criteria

**Week 1**: Clone workspace, describe a task in plain English, system detects type, routes correctly, completes pipeline, produces output. Human edits one intermediate artifact. Next stage picks up the edit correctly.

**Month 1**: System has run 20+ times across 3+ task types. Improvement engine has proposed and applied 5+ handler modifications. Quality metrics show measurable improvement on mature task types. One new task type has been auto-discovered and scaffolded from a PRD. The gap registry has a history that reads like a learning record.

**Month 3**: Drop an agent into any stage folder without providing external context. It orients correctly from the MANIFEST alone and completes its task. The fractal test passes at every depth in the tree.

**Month 6**: System handles task types it was never explicitly designed for. Quality on mature workspaces approaches human-edited baseline. Pending.txt entries from 3 months ago are traceable to the handler improvements they generated. The runner has completed 50+ autonomous gap-resolution cycles.

**The only success criterion that actually matters**: The system's outputs improve without you touching the code.

The system is alive when its outputs improve without you touching the code. Every other metric is a leading indicator of that one.

---

## Appendix A: The Packet Analogy — Complete Mapping

| IP Packet Concept | ALWS Equivalent |
|---|---|
| Header | MANIFEST.md |
| Payload | File contents |
| Router | Agent reading MANIFEST + following routing rules |
| Hop-by-hop routing | MANIFEST navigation (one level at a time) |
| TTL (Time To Live) | finalized.flag (prevents infinite retry loops) |
| Fragmentation | Stage pipeline (large tasks broken into stages) |
| Reassembly | Global synthesis (fragments recombined at GLOBAL/) |
| ACK | Verification gate in stage contract |
| Network topology | Folder tree |
| DNS | REGISTRY.md (name → location mapping) |

The internet scales because every router understands the same packet format regardless of payload content. ALWS scales because every agent understands the same MANIFEST format regardless of what the folder contains.

---

## Appendix B: Gap Type Decision Tree

```
New gap detected
      ↓
Does a concept node exist?
  No → Not a gap yet. Route to intake.
  Yes ↓
Does the concept have a composition table?
  No → missing_composition (Type 1)
  Yes ↓
Are the composition's components in the graph?
  No → shallow_node (Type 3)
  Yes ↓
Are there two nearby nodes with shared sub-components but no direct edge?
  Yes → missing_bridge (Type 2)
  No → Not a gap. Node is complete.
```

---

## Appendix C: Conversation Lineage

This PRD synthesizes across four source conversations:

1. **Prompt engineering and recursive prompt creation** (`148ec761`) — The research directive that established the meta-prompting conceptual foundation
2. **Comprehensive prompt engineering and multi-agent research** (`30e5b295`) — The deep research phase
3. **Thanks to Twinski** (`b4031299`) — The original metaA–metaG pipeline with orchestrator, PreAgents, and folder-based state machine; the composition knowledge model; the Ur-Prompt development; the fractal MANIFEST insight; the TCP/IP analogy; the gap detection engine; the runner; the PRD intake pipeline; and the analysis of Jake's ICM/MWP system
4. **Infinite capabilities of agents** (`eafe1ab4`) — Agent identity and continuity context
