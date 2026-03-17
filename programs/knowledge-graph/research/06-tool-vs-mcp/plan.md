# Study 06 — Tool_use vs MCP Server for file-selector

Status: designed
ADR resolved: ADR-008
Priority: CRITICAL — blocks file-selector implementation

---

## Research Question

Should file-selector be implemented as:
(A) A Claude tool_use function — passed in the `tools` array of each API call, backed by Python
(B) An MCP (Model Context Protocol) server — a persistent process Claude connects to

---

## Why This Matters

This is the most important architectural decision for the project's usability.

**Tool_use path:**
- File-selector is a JSON schema + a Python function that Claude calls
- Requires: every API call includes the tool definition
- Works: in any Claude API context (API, Claude Code, etc.)
- Limitation: tool must be re-defined every conversation

**MCP path:**
- File-selector is a persistent server process
- Claude discovers it via MCP config
- Works: automatically available in Claude Code and any MCP-compatible client
- Limitation: requires MCP server running; more complex to set up

---

## Hypothesis

**H1:** MCP is the better long-term architecture because file-selector should be
persistent and reusable across conversations, not re-defined each time.

**H2:** Tool_use is the better first implementation because it requires no server
infrastructure and can be tested immediately with the Claude API.

**H3:** The two approaches are functionally equivalent for the user experience
(both allow the AI to call file-selector mid-conversation).

---

## Evaluation Dimensions

### Dimension 1 — Developer Experience
How hard is it to build and test each approach?

Tool_use:
- Write Python function: ~50 lines
- Define JSON schema: ~20 lines
- Test: call via Claude API with tools=[] parameter
- Total setup: ~2 hours

MCP:
- Write MCP server (uses MCP Python SDK): ~150 lines
- Configure Claude Code settings.json: 10 lines
- Test: run server, open Claude Code, verify tool appears
- Total setup: ~4-6 hours

### Dimension 2 — Operational Simplicity
How easy is it to use day-to-day?

Tool_use: every session must include the tool definition in the API call.
If using Claude Code directly (not API), tool_use is not available.

MCP: start the server once (or configure it to auto-start).
Available in Claude Code automatically. No per-call configuration.

### Dimension 3 — Persistence and State
Can file-selector maintain state across calls?

Tool_use: stateless per call. Python function runs, returns, exits. No memory between calls.
MCP: server is persistent. Can cache the vector index, keep ticker.log file handle open,
maintain session state between calls.

### Dimension 4 — Portability
Can other tools / agents use file-selector without extra work?

Tool_use: any Claude API consumer can use it by including the tool definition.
MCP: any MCP-compatible client can use it. Broader ecosystem.

### Dimension 5 — Ticker Logging Correctness
Does ticker.log get written correctly in both modes?

Tool_use: Python function writes to ticker.log. Simple. No server needed.
MCP: MCP server writes to ticker.log. File locking may be needed if multiple clients.

---

## Recommended Test

### Test A — Tool_use prototype
1. Write a minimal file_selector.py function (read file, append to ticker, return content)
2. Define the tool schema (JSON)
3. Run a test API call with `tools=[file_selector_schema]`
4. Have Claude navigate the 20-file test set via tool calls
5. Verify: files returned correctly, ticker.log has correct entries

### Test B — MCP server prototype
1. Write file_selector_server.py using mcp Python library
2. Configure Claude Code to use the server
3. Open a new Claude Code conversation — verify file_selector tool appears
4. Have Claude navigate the same 20-file test set
5. Verify: same quality as Test A

### Comparison
- Test A setup time vs Test B setup time
- Navigation quality: same? (should be)
- Ticker correctness: same? (should be)
- Operational friction: which is easier to use repeatedly?

---

## Decision Matrix

| Criterion | Weight | Tool_use | MCP |
|-----------|--------|---------|-----|
| Build speed | 2x | — | — |
| Operational simplicity (Claude Code use) | 3x | — | — |
| State persistence | 2x | — | — |
| Portability | 1x | — | — |
| Ticker correctness | 3x | — | — |

Score each 1-5. Weighted sum determines recommendation.

---

## The Hybrid Path (expected conclusion)

Build tool_use first. It runs in any context and requires no server.
Document the MCP upgrade path but don't build it yet.

If Claude Code integration becomes important (likely), migrate to MCP.
The Python function is the same either way — only the wrapper changes.

ADR-008 update if hybrid wins: "Tool_use for initial implementation. MCP upgrade
documented but deferred until Claude Code integration is a requirement."
