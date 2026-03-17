# _planning — Architecture Workspace

## What This Is
Where decisions get made before code gets written. An agent does not touch
`programs/` without first checking whether a relevant decision exists here.

---

## What Lives Here

| Folder / File | Purpose |
|---------------|---------|
| `adr/` | Architecture Decision Records — permanent log of what was decided and why |
| `system-design/` | Data flow diagrams, service boundaries, sequence diagrams |
| `roadmap.md` | What is built, what is in progress, what is planned — in that order |
| `standards.md` | Project-wide technical standards (create when you have 3+ programs) |

---

## Task Routing

| Your Task | Do This |
|-----------|---------|
| Make a new architectural decision | Create `adr/[NNN]-[slug].md` using the ADR template below |
| Check if a decision already exists | Read `adr/` file list, then read the relevant ADR |
| Document system design | Create or update files in `system-design/` |
| Check project status | Read `roadmap.md` |
| Define cross-program technical standards | Create or update `standards.md` |

---

## ADR Template

```markdown
# ADR-[NNN]: [Decision Title]

## Status
[proposed | accepted | assumption | superseded by ADR-NNN]

## Context
[What situation forced this decision. One paragraph max.]

## Decision
[What was decided. One sentence if possible.]

## Consequences
[What becomes easier. What becomes harder. What is now off the table.]

## Alternatives Considered
[Only required for accepted ADRs. Leave blank for assumption ADRs.]
```

**Status meanings:**
- `accepted` — stated or clearly implied in the PRD. Binding for all programs.
- `assumption` — inferred to fill a PRD gap. **Blocks building.** A human must change this
  to `accepted` before any program that depends on this decision can move to `specced`.
- `proposed` — written but not yet reviewed. Not yet binding.
- `superseded by ADR-NNN` — replaced. Never deleted.

**Rules for ADRs:**
- Never delete an ADR. Supersede it with a new one.
- If code contradicts an `accepted` ADR, the code is wrong — not the ADR.
- An `assumption` ADR is not a placeholder. It is a hard stop signal.

---

## What Planning Produces

| Output | Goes To |
|--------|---------|
| Accepted ADRs | Binding constraints for `programs/*/CONTEXT.md` |
| System design docs | Reference for build agents in `programs/*/` |
| Contracts | `../shared/contracts/` (not here — planning defines them, shared/ holds them) |
| Roadmap | `roadmap.md` — build order and live status for anyone entering this project |
