# Roadmap — project-beta

## Build Order

Single-program project — no dependency graph required.

| Build Order | Program | Depends On | Status |
|-------------|---------|-----------|--------|
| 1 | `programs/cli/` | none | scaffold |

---

## Status

| Program | Status | Blocking | Notes |
|---------|--------|---------|-------|
| programs/cli/ | scaffold | assumption ADRs (if any) | — |

---

## Next Actions

1. Resolve all `assumption` ADRs in `_planning/adr/` — change to `accepted` after human review
2. Run `{root}/_meta/spec-review.md` on `programs/cli/`
3. On PASS: move cli to `specced`, begin build

---

## Completion Gate

Project moves to `complete` when:
- `programs/cli/` has status `complete`
- No open blocking gaps in `_meta/gaps/CONTEXT.md`
