# Dreamworld — Task Router

**Status**: reference — source of truth for game_engine requirements. Read-only.

---

## What To Do Here

| Task | Action |
|------|--------|
| Extract requirements for game_engine | Read `prd.txt`, cross-reference with `programs/game_engine/PRD.md` |
| Understand the phased build order | Read `implementationplans/implementationMaster.txt` |
| Resolve a game_engine ADR from requirements | Find the relevant PRD section in `prd.txt` |
| Check phase boundaries and shippable milestones | `implementationMaster.txt` §0 and phase headings |

## What NOT To Do Here

- Do not treat `prd.txt` as active — it is the vision source, not a build ticket
- Do not build inside this folder
- Do not update `prd.txt` directly — if the vision evolves, create a `prd-v2.txt`

## Connection to Active Projects

- `programs/game_engine/` — the active implementation of this vision; game_engine's PRD.md was derived from this document
- `programs/ELEV8/` — the hackathon prototype that attempted an earlier version of this vision
