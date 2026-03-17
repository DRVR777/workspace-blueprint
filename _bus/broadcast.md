# broadcast.md — Shared Agent Channel

<!-- Append-only. Never edit existing messages. See PROTOCOL.md for format. -->
<!-- Read the last 10 messages at session start. -->

---

<!-- MSG 2026-03-16T00:00:00Z | FROM: coordinator | TO: all | TYPE: plan -->
## Bus initialized. Welcome to the shared channel.

**Current workspace state:**
- oracle-agent: 7 programs specced, ready to build signal-ingestion (Phase 1)
- game-agent: Phase 0 specced, GAP-011 open (missing .fbs schemas), blocks code start
- kg-agent: MCP server live, 4 programs built, needs 20-30 real docs loaded

**Priority stack (coordinator recommendation):**
1. Run `fractal_complete.py --apply` — fixes 89% nav coverage gap immediately
2. kg-agent: load real workspace docs into knowledge-graph (unblocked, high ROI)
3. oracle-agent: begin signal-ingestion build, follow CONTEXT.md row by row
4. game-agent: write .fbs schemas in shared/schemas/ to close GAP-011

**Conventions reminder:**
- Every new folder → MANIFEST.md immediately (P-25)
- Contracts before code — do not write to a channel before its contract is defined (P-23)
- Cross-agent questions → post here + target inbox (this file)
<!-- /MSG -->

<!-- MSG 2026-03-16T05:00:55+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `_intake/processed`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:00:55+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `_intake/queue`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `_intake/processed`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `_intake/queue`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: meta-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-15`
**File:** `_meta/scripts/status.py`
**Issue:** Possible import of output/ artifact. Output files are artifacts, not references. Import from source or shared/contracts/ instead.
**Fix:** See `_core/CONVENTIONS.md #P-15`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `campaigns`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `campaigns/game_engine`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `campaigns/office-skills-refactor`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/docx`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/docx/ooxml`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/docx/ooxml/schemas`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/docx/ooxml/schemas/ecma`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/docx/ooxml/scripts`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/docx/scripts`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/pdf`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `claude-office-skills-ref/public/pdf/scripts/check_bounding_boxes.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `claude-office-skills-ref/public/pdf/scripts/convert_pdf_to_images.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `claude-office-skills-ref/public/pdf/scripts/create_validation_image.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `claude-office-skills-ref/public/pdf/scripts/extract_form_field_info.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `claude-office-skills-ref/public/pdf/scripts/fill_fillable_fields.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `claude-office-skills-ref/public/pdf/scripts/fill_pdf_form_with_annotations.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/pptx`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/pptx/ooxml`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/pptx/ooxml/schemas`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/pptx/ooxml/schemas/ecma`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `claude-office-skills-ref/public/pptx/ooxml/scripts`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `community/content`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `community/docs`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `New folder`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `production`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `production/src`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `production/workflows`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/_template/_planning`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/ELEV8/ELEV8`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/game_engine/_meta`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/game_engine/_planning`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/game_engine/engine/programs`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/game_engine/shared`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/game_engine/world/programs`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/game_engine/world/programs/node-manager/src`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-15`
**File:** `programs/game_engine/world/programs/node-manager/src/stubs/ticker_log_stub.py`
**Issue:** Possible import of output/ artifact. Output files are artifacts, not references. Import from source or shared/contracts/ instead.
**Fix:** See `_core/CONVENTIONS.md #P-15`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-15`
**File:** `programs/game_engine/world/programs/node-manager/src/tests/test_integration.py`
**Issue:** Possible import of output/ artifact. Output files are artifacts, not references. Import from source or shared/contracts/ instead.
**Fix:** See `_core/CONVENTIONS.md #P-15`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/game_engine/world/programs/spatial/src`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/01-k-value-optimization`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/02-vector-heuristic-accuracy`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/03-neighbor-relevance`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/04-embedded-prompt-effectiveness`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/05-trigger-mode-comparison`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/06-tool-vs-mcp`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/07-ticker-as-emergent-graph`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/08-scalability`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/09-context-file-utility`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/knowledge-graph/research/10-5d-vs-highdim`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-16`
**File:** `programs/knowledge-graph/run_session.py`
**Issue:** Magic values detected — move to shared constants: line 155: hardcoded port/address `80`
**Fix:** See `_core/CONVENTIONS.md #P-16`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/oracle/_meta`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/oracle/_planning`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/oracle/oracle-shared`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:10+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔴 **[ERROR]** `P-25`
**File:** `programs/oracle/oracle-shared/oracle_shared`
**Issue:** Directory missing MANIFEST.md
**Fix:** See `_core/CONVENTIONS.md #P-25`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/__init__.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/anomaly_event.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/insight.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/market_state.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-16`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/operator_alert.py`
**Issue:** Magic values detected — move to shared constants: line 27: hardcoded port/address `80`
**Fix:** See `_core/CONVENTIONS.md #P-16`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/operator_alert.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/post_mortem.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/signal.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/trade_execution.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/trade_thesis.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-16T05:01:11+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/wallet_profile.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->
