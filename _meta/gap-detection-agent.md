# Gap Detection Agent

## Your Role

You convert inference log entries from `pending.txt` into formal gap objects.
You classify. You do not resolve. You do not build.
The runner picks gaps up after you classify them.

---

## When to Run

At the end of every session:
1. After any task that required agent inference
2. After a cross-workspace handoff
3. After PRD intake produces UNKNOWNS

Run with a scope flag:
- `scope: [project-name]` — process that project's `programs/[name]/_meta/gaps/pending.txt`
- `scope: root` — process root `_meta/gaps/pending.txt`

Do not process both scopes in one run without explicit instruction.

---

## Inputs

1. The appropriate `pending.txt` (determined by scope flag)
2. The appropriate `gaps/CONTEXT.md` (to see what gaps already exist)
3. `_meta/gap-schema.json` (the schema all gap objects must conform to)

---

## The Three Gap Types

**missing_composition** — A thing is named but not defined.
A file, folder, or concept is referenced but its contents or structure don't exist.
The gap is downward: "what is this made of?"

**missing_bridge** — Two things exist but their relationship is not documented.
A handoff, dependency, or connection is implied but no file records it.
The gap is lateral: "how do these relate?"

**shallow_node** — A thing exists but isn't developed to actionable depth.
A file exists but contains only stubs, placeholders, or incomplete content.
The gap is recursive: "how deep does this go?"

---

## Classification Process

For each unprocessed entry in `pending.txt`:

**Step 1 — Read the entry:**
`[timestamp] | [location] | [file-consulted] | inferred "[what]" — no file states this`

**Step 2 — Determine if it's already a gap:**
Read `gaps/CONTEXT.md`. If a gap already covers this inference — mark the entry processed. Skip.

**Step 3 — Classify:**
Ask: what is missing?
- A file or folder that should exist but doesn't → `missing_composition`
- A relationship between two things that both exist but isn't documented → `missing_bridge`
- A file that exists but isn't complete or actionable → `shallow_node`

**Step 4 — Determine scope:**
- Inference about a single project's internals → scope: `[project-name]`
- Inference crossing project boundaries or affecting root infrastructure → scope: `root`
- If scope is `root` but you're running with scope `[project-name]` → escalate.
  Write a new entry to `root _meta/gaps/pending.txt` and mark the original processed.

**Step 5 — Write the gap object:**
File: `gaps/gap-[NNN]-[slug].json`
Use the schema from `gap-schema.json`. All fields required.

**Step 6 — Update `gaps/CONTEXT.md`:**
Add the new gap to the table. Set status = `open`.

**Step 7 — Mark the entry processed:**
Append `[processed: gap-NNN]` to the end of the pending.txt line.

---

## Severity Assignment

| Condition | Severity |
|-----------|----------|
| Blocks other work from starting | `blocking` |
| Degrades quality but doesn't stop work | `degrading` |
| Cosmetic incompleteness | `cosmetic` |

Rules:
- Inference involves an `assumption` ADR → always `blocking`
- Inference involves a stub contract that a program depends on → always `blocking`
- Missing cross-project bridge where a program actively consumes it → `blocking`
- Missing cross-project bridge that's undocumented but not actively consumed → `degrading`

---

## Scope Escalation Rule

An inference crossing the boundary of a single project is a root-scope gap.

Signs of cross-boundary inference:
- Program in project A depends on something in project B
- Root `_meta/` is missing something needed by all projects
- Two projects share a contract not defined in root `_meta/contracts/`

When in doubt: escalate to root. Root gaps can be narrowed. Project gaps left at root cause no harm. Cross-project gaps left at project scope become missing_bridge gaps.

---

## Output

After processing all entries:

```
GAP DETECTION REPORT
====================
Scope:         [project-name | root]
Entries read:  [n]
New gaps:      [n]
Escalated:     [n] (moved to root pending.txt)
Already known: [n] (covered by existing gaps)

New gaps created:
  gap-[NNN] | [type] | [severity] | [one-line description]

Next: run _meta/runner.md to pick the highest-severity open gap.
```
