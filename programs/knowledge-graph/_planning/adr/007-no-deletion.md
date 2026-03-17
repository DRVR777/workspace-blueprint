# ADR-007: File Deletion Policy — Deprecate, Never Delete

Status: accepted — 2026-03-14
Date: 2026-03-13

## Decision
Files in Data/ are never deleted. When content is obsolete: add `deprecated: true`
to metadata header and update the embedded prompt to redirect to the replacement.

## Rationale
- File numbers in ticker.log are permanent references — deleting a file breaks the log
- Other files may list a deprecated file in their neighbors list
- Ticker analysis (which files are accessed together) remains valid over time
- Zettelkasten principle: notes are never deleted, only superseded

## Deprecation Pattern
```yaml
---
filename: NNNN
vector: [0.x, 0.x, 0.x, 0.x, 0.x]
neighbors: [...]
deprecated: true
superseded_by: MMMM
---

<!-- EMBEDDED PROMPT -->
This file is deprecated. Read file[MMMM] instead.
Call file-selector("MMMM") now.
<!-- END EMBEDDED PROMPT -->
```

## Consequences
- Data/ grows monotonically — never shrinks
- index.md marks deprecated files with a [DEPRECATED] tag
- The proximity query in file-selector skips deprecated files by default
  (unless `include_deprecated=true` is passed)
- Ticker analysis shows deprecated files as historical nodes — useful for understanding
  how the knowledge base evolved

## Needs human validation
If storage becomes a concern, implement an archive system (move deprecated files to Data/archive/).
