# ADR-008: file-selector as a Claude Tool (tool_use)

Status: accepted — 2026-03-14 (tool_use for initial build; MCP migration is a named future path)
Date: 2026-03-13

## Decision
file-selector is implemented as a Claude tool — a JSON schema defining a function
the AI can call via tool_use. The tool is backed by a Python script that handles
file reads and ticker writes.

## What This Means
The tool definition (passed to the Claude API in the `tools` array):

```json
{
  "name": "file_selector",
  "description": "Read a document from the knowledge graph. Every call is logged to the global ticker. Use this to navigate the document graph.",
  "input_schema": {
    "type": "object",
    "properties": {
      "file_number": {
        "type": "string",
        "description": "4-digit file number to read (e.g. '0042'). Omit to use proximity query."
      },
      "query_vector": {
        "type": "array",
        "items": {"type": "number"},
        "description": "5-element vector [s, t, temp, c, conf] for proximity search. Omit to read by number."
      },
      "k": {
        "type": "integer",
        "description": "Number of neighbors to return in proximity query. Default 5.",
        "default": 5
      },
      "session_id": {
        "type": "string",
        "description": "Current session identifier for ticker logging."
      },
      "reason": {
        "type": "string",
        "description": "Why this file is being read (direct_read, neighbor_of_NNNN, proximity_query)."
      }
    },
    "oneOf": [
      {"required": ["file_number"]},
      {"required": ["query_vector"]}
    ]
  }
}
```

## The Backing Script
`programs/file-selector/src/file_selector.py`

When called with `file_number`:
1. Read `Data/file[NNNN].md`
2. Append to `Data/ticker.log`
3. Increment `access_count` in file metadata
4. Return file content as JSON: `{"file_number": "NNNN", "metadata": {...}, "prompt": "...", "content": "..."}`

When called with `query_vector`:
1. Load all vectors from `Data/index.md` or scan metadata headers
2. Compute distances to query_vector
3. Return k nearest as array of file content objects

## Consequences
- The AI can call this tool mid-conversation, just like any other Claude tool
- The tool is passed in the `tools` parameter of every API call in this session
- file-selector.py must be invokable as a subprocess or importable as a module
- MCP (Model Context Protocol) is the natural home for this — the file-selector
  can be exposed as an MCP tool so any Claude instance can use it
- For testing without the API: file-selector.py can be called from the command line

## Needs human validation
Confirm: should this be an MCP server or a standalone tool definition?
MCP enables reuse across sessions and projects. Standalone is simpler to build first.
