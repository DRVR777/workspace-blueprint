# Architecture Decision Records

One file per decision. Format: [NNN]-[slug].md
Number sequentially from 001. Never delete. Supersede instead.

Status legend:
- `accepted` — binding, derived from PRD explicit statements
- `assumption` — inferred to fill PRD gap — needs human validation before affected programs build
- `proposed` — written but not yet reviewed
- `superseded by ADR-NNN` — replaced, never deleted

When prd-intake.md runs, it auto-generates ADRs here:
- One `accepted` ADR per explicit PRD decision
- One `assumption` ADR per PRD unknown

Assumption ADRs block building. See `{root}/_meta/status-transitions.md`.
