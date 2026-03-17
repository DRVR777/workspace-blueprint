# ELEV8 — Task Router

**Status**: reference — no active development. Read-only unless extracting patterns.

---

## What To Do Here

| Task | Action |
|------|--------|
| Extract reusable patterns for game_engine | Read `ELEV8/analysis_report.md` §"What is reusable" |
| Understand the original architecture | Read `ELEV8/backend.pdf`, `ELEV8/dreamworld_backend_architecture.pdf` |
| Review what the frontend looked like | Open `ELEV8/images/` screenshots |
| Watch the live demo | `ELEV8/recordings/` — 3 recordings from 2026-03-08 and 2026-03-12 |
| Conduct failure analysis | Start with `analysis_report.md`, cross-reference against `dreamworld/prd.txt` |

## What NOT To Do Here

- Do not build new features inside this folder
- Do not update the PDFs — they are frozen artifacts
- Do not treat this as active source code — everything here is reference material

## Connection to Active Projects

- `programs/game_engine/` — the production rebuild; pulls patterns from this project
- `programs/dreamworld/` — the vision PRD; ELEV8 was the first attempt to realize it
