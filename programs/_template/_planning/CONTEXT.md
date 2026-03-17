# _planning — Architecture Workspace

## What This Is
Where decisions get made before code gets written.
An agent does not touch `programs/` without first checking whether a relevant decision exists here.

---

## What Lives Here

| Folder / File | Purpose |
|---------------|---------|
| `adr/` | Architecture Decision Records — permanent, never deleted, only superseded |
| `system-design/` | Data flows, service boundaries, component diagrams |
| `roadmap.md` | Build order — what gets built first and why |

---

## Task Routing

| Your Task | Do This |
|-----------|---------|
| Make a new architectural decision | Check adr/ first — if it exists, read it. If not, write a new ADR. |
| Document system design | Create or update files in system-design/ |
| Check build order or project status | Read roadmap.md |

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

**Rules:**
- Never delete an ADR. Supersede it with a new one.
- If code contradicts an `accepted` ADR, the code is wrong — not the ADR.
- An `assumption` ADR is not a placeholder. It is a hard stop signal.

---

## What Planning Produces

| Output | Goes To |
|--------|---------|
| Accepted ADRs | Binding constraints for `programs/*/CONTEXT.md` |
| System design docs | Reference for build agents in `programs/*/` |
| Contract definitions | `../shared/contracts/` (planning defines, shared/ holds) |
| Roadmap | Build order and status for anyone entering this project |
