# Ur-Prompt — Domain-Agnostic Workspace Orientation and Prompt Generator

---
id: ur-prompt-v1
type: layer-0-generator
domain: null
version: 1.0.0
---

## What This File Is

This is Layer 0. It knows nothing about what this workspace does.
It knows the shape of how workspaces are built here and how to generate
execution prompts for any gap in any workspace of this type.

Read this before reading CLAUDE.md. It makes every subsequent read faster.

---

## The Shape of This Workspace (Domain-Independent)

Every workspace in a system like this has the same skeleton:

```
[root]/
├── CLAUDE.md          — always-loaded map. Folders + naming rules.
├── CONTEXT.md         — task router. "Your task → go here."
├── MANIFEST.md        — envelope. "What this folder is + contains + needs + gives."
└── [workspace]/
    ├── CONTEXT.md     — workspace scope. "This task → load these files."
    ├── MANIFEST.md    — envelope for this workspace.
    ├── docs/          — stable reference knowledge. Load on demand.
    └── [stages]/      — files that move through a process.
        └── MANIFEST.md  — envelope for this stage.
```

Full orientation = 3 reads maximum:
1. MANIFEST.md at current depth (where am I, what do I contain)
2. Parent MANIFEST (how did I get here, what do I depend on)
3. Child MANIFEST of the folder I am routing into (where am I going)

---

## What Makes a Complete Execution Prompt (Domain-Independent)

Every task prompt has exactly 5 properties. A prompt missing any of them is
incomplete and will produce unpredictable output.

1. **POSITION** — Show the agent exactly where in the structure the task sits.
   What exists. What is adjacent. What is missing. Derived from MANIFEST + CONTEXT.

2. **SHAPE** — Define the exact output format the agent must produce.
   Tables, files, edges. The shape must be rejecting: bad outputs don't fit it.

3. **CONSTRAINT** — Remove degrees of freedom. Name what NOT to do.
   Always include: do not summarize, do not fill adjacent gaps, do not produce
   prose where structure is required.

4. **SCOPE** — Define exactly where this task ends. What counts as done.
   What counts as out of scope. One task per prompt.

5. **VERIFICATION** — Give the agent a self-check it can run before submitting.
   "Does your output satisfy the SHAPE defined above? If no, do not submit."

---

## Generating an Execution Prompt from a Gap Object

Given a gap object (from `_meta/gaps/`), generate a complete execution prompt:

**Step 1 — Read the gap object.**
Extract: gap_type, detected_in, description, proposed_expansion.

**Step 2 — Determine POSITION from gap_type:**
- missing_composition → Show the agent the concept that lacks decomposition.
  List what the graph already knows within 2 hops of this concept.
- missing_bridge → Show both concepts. Show their shared sub-components.
  State the bridge hypothesis.
- shallow_node → Show the concept. Show its current definition.
  State what depth would make it actionable.

**Step 3 — Determine SHAPE from gap_type:**
- missing_composition → Output: composition table (Component | Role | Concept ID)
  + "Novel Combination" section declaring what is non-obvious about how parts combine.
- missing_bridge → Output: edge declaration (From | To | Type | Mechanism)
  or indirect path with new mediating node + its composition table.
- shallow_node → Output: expanded definition with worked examples, edge cases,
  and explicit "this is actionable when:" statement.

**Step 4 — Write CONSTRAINT:**
Always include: "Do not summarize. Do not explain history. Do not fill adjacent gaps.
Do not produce prose where structure is required."
Add gap-specific constraints from the proposed_expansion field.

**Step 5 — Write SCOPE:**
"Fill this gap: [gap description]. Stop when the gap object's proposed_expansion
is satisfied. Do not address gaps in adjacent folders."

**Step 6 — Write VERIFICATION:**
"Before submitting: Does your output match the SHAPE defined in Step 3?
Does it close the gap described in POSITION? If either answer is no, do not submit."

**Step 7 — Assemble.**
Combine Steps 2–6 into a single prompt. This prompt is ready to execute.

---

## The Inference Logging Rule

During any task in this workspace: if you make a decision not explicitly supported
by a loaded file, append to `_meta/gaps/pending.txt`:

```
[ISO-8601-timestamp] | [workspace] | [file-consulted] | inferred "[what]" — no file states this
```

This is how the system learns what it doesn't know.

---

## The One Test

If this file mentions anything domain-specific (blog posts, video production,
Acme, DevRel, or any other content category), it has violated Layer 0.
The test: "Would this file work equally well for a legal research system,
a protein folding database, or a Byzantine tax law archive?"
If yes, the abstraction is real. If no, domain knowledge has leaked upward.
