---
name: ELEV8
type: project
status: reference
version: 1.0
parent: programs/
---

# ELEV8 — Dreamworld Hackathon Prototype

**What this is**: The original hackathon build of Dreamworld. A multi-application ecosystem
(3D Museum + Graph Visualizer + AI Chatbot) built under time pressure. Kept as a reference
for failure analysis and pattern extraction, not as an active project.

**What it contains**:

| Folder/File | Purpose |
|-------------|---------|
| `ELEV8/` | All source artifacts from the hackathon |
| `ELEV8/prd.pdf` | Original hackathon PRD |
| `ELEV8/backend.pdf` | Backend architecture spec |
| `ELEV8/analysis_report.md` | Post-mortem: what to keep, what to rebuild |
| `ELEV8/dreamworld_*.pdf` | Frontend visual references and structure |
| `ELEV8/howToModularDB.txt` | Notes on database modularization |
| `ELEV8/images/` | Screenshots from the live prototype |
| `ELEV8/recordings/` | Screen recordings of the working demo |

## Routing

| You want to... | Go to |
|----------------|-------|
| Understand what was built | `ELEV8/analysis_report.md` |
| See the original requirements | `ELEV8/prd.pdf` |
| Extract patterns for game_engine | `ELEV8/analysis_report.md` → "What is reusable" section |
| See the backend architecture | `ELEV8/backend.pdf` |

## Key Findings (from analysis_report.md)

- **Keep**: AI Mutation Architecture (prompt → JSON → 3D state), System/World pattern
- **Discard**: Vanilla JS architecture — not scalable to Dreamworld scope
- **Rebuild path**: Next.js + React Three Fiber unified app for hackathon; proper NEXUS engine for production
