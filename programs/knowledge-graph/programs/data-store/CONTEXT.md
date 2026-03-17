# programs/data-store — Task Router

## What This Program Is
Owns and manages the Data/ folder. Creates files with correct naming and format.
Maintains index.md. Nothing else touches file creation — only data-store.

---

## Before Writing Any Code
Read ADR-001 (file format) and ADR-007 (no deletion policy).
Read shared/contracts/file-record.md — this is what data-store produces.
Run spec-review when all contracts are non-stub.

---

## Task Routing

| Your Task | Load These | Skip These |
|-----------|-----------|------------|
| Create a new file | MANIFEST.md, shared/contracts/file-record.md | indexer, context-builder src/ |
| Validate file format | _planning/file-format-spec.md | other programs |
| Update index.md | shared/contracts/index-entry.md | vector specs |
| Understand embedded prompt template | _planning/file-format-spec.md §Embedded Prompt | — |

---

## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| File format implemented | Sample file0001.md | approve format / revise |
| Counter logic done | Create 3 files, show numbering | approve / fix gaps |

## Audit
Before committing to output/:
- [ ] file0001.md exists in Data/ and matches file-format-spec.md exactly
- [ ] index.md has one row per file in Data/
- [ ] Counter produces sequential numbers with no gaps
- [ ] No existing file was modified by the creation of a new file

---

## Inputs
- File content (text)
- Optional: custom embedded prompt override

## Outputs
- `Data/file[NNNN].md` — new file with full metadata header
- `Data/index.md` — updated with new entry
