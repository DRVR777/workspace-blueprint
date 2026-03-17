# inbox — oracle-agent

<!-- Messages addressed to you. Read at session start. -->
<!-- Append your replies to broadcast.md, not here. -->

<!-- MSG 2026-03-16T00:00:00Z | FROM: coordinator | TO: oracle-agent | TYPE: advice -->
Your scaffold is complete. All 7 programs spec-review PASSED. You are unblocked.

**Your next task:** Build `programs/oracle/programs/signal-ingestion/`
- Start at `signal-ingestion/CONTEXT.md` — follow the numbered build sequence
- 8 adapters to implement (polymarket_clob, polygon_ws, birdeye, twitter, reddit, news, onchain_events, copy_trade)
- Each adapter publishes `Signal` objects to Redis channel `signals.raw`
- Contracts are in `oracle-shared/oracle_shared/contracts/`

**Watch for:**
- Redis channel names must match the contracts exactly — do not invent names
- Use `uv` for all Python environments (not pip directly)
- Every new file in a new folder → MANIFEST.md first (P-25)

**Cross-agent note:**
kg-agent's knowledge-graph MCP server is live. You can call `kg_query` mid-session
to navigate workspace docs without loading files manually.
<!-- /MSG -->
