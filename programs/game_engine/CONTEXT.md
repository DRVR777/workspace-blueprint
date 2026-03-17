# game_engine — Task Router

*Your task → go here*

| If you want to... | Go to |
|-------------------|-------|
| Understand the full requirements | `PRD.md` |
| Work on world/spatial architecture | `world/MANIFEST.md` |
| Work on the local rendering engine | `engine/MANIFEST.md` |
| Make an architectural decision | `_planning/adr/` |
| See the build order | `_planning/roadmap.md` |
| Find open gaps | `_meta/gaps/pending.txt` |
| Understand subsystem interfaces | `shared/contracts/` |
| Read the campaign tracker | `campaigns/game_engine/clarification.md` |
| Read the north-star vision | `campaigns/game_engine/progress/perfect_final_output.md` |
| Understand ELEV8 failures | `PRD.md` Appendix A |
| Understand knowledge-graph application | `PRD.md` Appendix B |
| Check what decisions are open | `PRD.md` Appendix C |

**Load rules**:
- Always load `MANIFEST.md` first
- Load `PRD.md` only when you need deep requirements detail
- Load a subsystem's MANIFEST before working in that subsystem
- Never load more than one subsystem's internals at once unless explicitly crossing boundaries
