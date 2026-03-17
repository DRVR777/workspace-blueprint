# Contract: ticker-entry

Status: defined
Produced By: file-selector (writes on every read)
Consumed By: context-builder (triggers on new entries), graph-builder (analyzes patterns)

---

## Shape

One line per entry in Data/ticker.log.

### Text format (in file)
```
[ISO-8601] | [file_number] | [session_id] | [reason]
```

Example:
```
2026-03-13T14:22:01Z | 0042 | session-abc123 | direct_read
2026-03-13T14:22:15Z | 0031 | session-abc123 | neighbor_of_0042
2026-03-13T14:22:31Z | 0042 | session-abc123 | context_built
```

### Parsed object
```json
{
  "timestamp": "2026-03-13T14:22:01Z",
  "file_number": "0042",
  "session_id": "session-abc123",
  "reason": "direct_read"
}
```

## Valid Reasons

| Reason | Written by | Meaning |
|--------|-----------|---------|
| `direct_read` | file-selector | AI called file-selector explicitly with this number |
| `neighbor_of_[N]` | file-selector | AI read this because it's a neighbor of file N |
| `proximity_query` | file-selector | AI found this via 5D proximity search |
| `revisit` | file-selector | AI has read this file before in this session |
| `context_built` | context-builder | context-builder wrote ctx-NNNN.md for this file |
| `indexed` | indexer | indexer assigned vector + neighbors to this file |

## Parsing Rules
- Lines beginning with `#` are comments — skip
- Empty lines — skip
- Split on ` | ` (space-pipe-space), exactly 3 splits
- If a line cannot be parsed: skip and log warning, do not crash
