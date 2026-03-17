# Contract: file-record

Status: defined
Produced By: data-store (writes), file-selector (reads and returns), indexer (updates metadata fields)
Consumed By: file-selector (returns to AI), indexer (reads for computation), context-builder (reads for ctx generation)

---

## Shape

The full representation of a Data/ file as a structured object.

```json
{
  "file_number": "0001",
  "metadata": {
    "filename": "0001",
    "vector": [0.8, 0.3, 0.2, 0.9, 0.9],
    "neighbors": ["0031", "0019", "0051", "0004", "0072"],
    "context_file": "Data/ctx-0001.md",
    "created": "2026-03-13",
    "last_indexed": "2026-03-13T14:22:01Z",
    "access_count": 7,
    "deprecated": false,
    "superseded_by": null
  },
  "embedded_prompt": "[full text of embedded prompt between comment markers]",
  "content": "[full document content below the embedded prompt]",
  "raw": "[complete file contents as string]"
}
```

## Field Definitions

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| file_number | string | yes | Zero-padded 4 chars |
| metadata.filename | string | yes | Same as file_number |
| metadata.vector | [float\|null] × 5 | yes | null before indexing |
| metadata.neighbors | string[] | yes | [] before indexing |
| metadata.context_file | string | yes | Path: Data/ctx-NNNN.md |
| metadata.created | date string | yes | ISO date YYYY-MM-DD |
| metadata.last_indexed | datetime string\|null | yes | null before first index |
| metadata.access_count | int | yes | Starts at 0 |
| metadata.deprecated | bool | yes | Default false |
| metadata.superseded_by | string\|null | yes | null if not deprecated |
| embedded_prompt | string | yes | May be empty string if no prompt section |
| content | string | yes | May be empty |
| raw | string | yes | Complete file text |

## Null Vector Handling
If `metadata.vector` contains any null values, the consumer should:
- file-selector: include in return value as-is; trigger indexing request in ticker
- indexer: this is a file that needs indexing — process it
- context-builder: skip position interpretation; write ctx file without My Position section
