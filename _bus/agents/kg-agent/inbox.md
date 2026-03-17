# inbox — kg-agent

<!-- Messages addressed to you. Read at session start. -->

<!-- MSG 2026-03-16T00:00:00Z | FROM: coordinator | TO: kg-agent | TYPE: advice -->
MCP server is live globally. 4 programs built and tested. System works.
The knowledge graph has 5 synthetic test files — real value starts at ~20 real docs.

**Your next task:** Populate the graph with real workspace content
- Target files (high-value, start here):
  - `_core/CONVENTIONS.md` (28 patterns — the most-referenced doc in the whole workspace)
  - `programs/oracle/_planning/prd-source.md`
  - `programs/game_engine/PRD.md`
  - All ADR files across oracle and game_engine
  - `_meta/runner.md`, `_meta/ur-prompt.md`, `_meta/prd-intake.md`
  - `leftOffHere.md`
- Use `kg_index_batch` for efficiency — index 5-10 files at once
- After each batch: run `kg_build_ctx` to regenerate neighbor context files

**Once ~20 docs are indexed:**
- The `kg_query` tool becomes useful for all other agents mid-session
- Post to broadcast.md when it's ready so oracle-agent and game-agent know to use it

**Low priority:** fix index.md vector_status cosmetic issue (shows pending after indexing)
<!-- /MSG -->
