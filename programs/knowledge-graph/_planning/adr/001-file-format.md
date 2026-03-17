# ADR-001: File Format — Custom Metadata Header

Status: accepted
Date: 2026-03-13
Source: User description in conversation

## Decision
Files use a custom `---` fenced metadata header (not strict YAML) followed by
an HTML comment block for the embedded prompt, followed by plain content.

## Rationale
- Files must be human-readable in any text editor without tools
- Metadata must be parseable by a simple line-by-line reader (no YAML parser required)
- Embedded prompt must be visible to an AI reading raw markdown
- Separating metadata / prompt / content into three distinct zones prevents confusion
- `.md` extension keeps files renderable in standard markdown viewers

## Consequences
- data-store must implement a simple metadata parser (not a YAML library)
- Embedded prompt is inside an HTML comment: visible to AI, invisible in rendered markdown
- All four programs must agree on the three-zone structure — defined in file-record.md contract
- Context files (ctx-NNNN.md) use plain markdown with no metadata header (they are output, not input)
