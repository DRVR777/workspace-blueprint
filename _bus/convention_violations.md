# convention_violations.md — Auto-Detected Violations

<!-- Auto-appended by convention_checker.py. Never edit manually. -->
<!-- Agents: read this at session start. Fix any violations in your domain. -->


<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: meta-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-15`
**File:** `_meta/scripts/status.py`
**Issue:** Possible import of output/ artifact. Output files are artifacts, not references. Import from source or shared/contracts/ instead.
**Fix:** See `_core/CONVENTIONS.md #P-15`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-15`
**File:** `programs/game_engine/world/programs/node-manager/src/stubs/ticker_log_stub.py`
**Issue:** Possible import of output/ artifact. Output files are artifacts, not references. Import from source or shared/contracts/ instead.
**Fix:** See `_core/CONVENTIONS.md #P-15`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: game-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-15`
**File:** `programs/game_engine/world/programs/node-manager/src/tests/test_integration.py`
**Issue:** Possible import of output/ artifact. Output files are artifacts, not references. Import from source or shared/contracts/ instead.
**Fix:** See `_core/CONVENTIONS.md #P-15`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: kg-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-16`
**File:** `programs/knowledge-graph/run_session.py`
**Issue:** Magic values detected — move to shared constants: line 155: hardcoded port/address `80`
**Fix:** See `_core/CONVENTIONS.md #P-16`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/__init__.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/anomaly_event.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/insight.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/market_state.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-16`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/operator_alert.py`
**Issue:** Magic values detected — move to shared constants: line 27: hardcoded port/address `80`
**Fix:** See `_core/CONVENTIONS.md #P-16`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/operator_alert.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/post_mortem.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/signal.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/trade_execution.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/trade_thesis.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🔵 **[INFO]** `STRUCT`
**File:** `programs/oracle/oracle-shared/oracle_shared/contracts/wallet_profile.py`
**Issue:** Python module missing top-level docstring. Add a brief description of what this module does.
**Fix:** See `_core/CONVENTIONS.md #STRUCT`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: oracle-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-16`
**File:** `programs/oracle/programs/signal-ingestion/signal_ingestion/__main__.py`
**Issue:** Magic values detected — move to shared constants: line 33: hardcoded port/address `6379`
**Fix:** See `_core/CONVENTIONS.md #P-16`
<!-- /MSG -->

<!-- MSG 2026-03-18T17:34:53+00:00 | FROM: convention-checker | TO: unknown-agent | TYPE: convention-violation -->
🟡 **[WARN]** `P-15`
**File:** `programs/watcher/watcher.py`
**Issue:** Possible import of output/ artifact. Output files are artifacts, not references. Import from source or shared/contracts/ instead.
**Fix:** See `_core/CONVENTIONS.md #P-15`
<!-- /MSG -->
