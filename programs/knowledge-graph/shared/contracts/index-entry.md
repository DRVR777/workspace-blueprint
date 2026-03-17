# Contract: index-entry

Status: defined
Produced By: data-store (writes on file creation, updates on index/deprecation)
Consumed By: file-selector (scans for proximity query), indexer (finds un-indexed files)

---

## Shape

One row per file in Data/index.md.

### Text format (in index.md table)

```markdown
| file0001 | 2026-03-14 | indexed | First sentence of document content. |
| file0002 | 2026-03-14 | pending | First sentence of document content. |
| file0003 | 2026-03-14 | deprecated | First sentence of document content. [DEPRECATED → 0004] |
```

### Parsed object

```json
{
  "file_number": "0001",
  "created": "2026-03-14",
  "vector_status": "indexed",
  "summary": "First sentence of document content."
}
```

## Field Definitions

| Field | Type | Values | Notes |
|-------|------|--------|-------|
| file_number | string | "0001"–"9999" | Zero-padded, no "file" prefix in parsed form |
| created | date string | ISO date | Set by data-store at creation |
| vector_status | enum | `pending`, `indexed`, `deprecated` | Updated by indexer and data-store |
| summary | string | free text | First sentence of content; truncated at 100 chars |

## Update Rules

- `pending` → `indexed`: indexer writes vector + neighbors to file metadata, then updates this row
- any → `deprecated`: data-store sets when `deprecated: true` is written to file metadata
- summary is written once at creation; not updated when content changes
- Rows are never deleted (mirrors no-deletion policy from ADR-007)
