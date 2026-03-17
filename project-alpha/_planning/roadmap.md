# Roadmap — project-alpha

## Build Order

Programs must build in dependency order per `_meta/status-transitions.md`.
A program cannot move to `specced` if any contract it *consumes* is produced by
a program still at `scaffold`.

| Build Order | Program | Depends On | Status |
|-------------|---------|-----------|--------|
| 1 | `programs/api/` | none | scaffold |
| 2 | `programs/frontend/` | `programs/api/` (via shared/contracts/) | scaffold |

---

## Status

| Program | Status | Blocking | Notes |
|---------|--------|---------|-------|
| programs/api/ | scaffold | assumption ADRs (if any), stub contracts | — |
| programs/frontend/ | scaffold | programs/api/ must be specced first | — |

---

## Next Actions

1. Resolve all `assumption` ADRs in `_planning/adr/` — change to `accepted` after human review
2. Define shapes for all stub contracts in `shared/contracts/`
3. Run `_meta/spec-review.md` on `programs/api/` first (no dependencies)
4. On PASS: move api to `specced`, begin build
5. Run `_meta/spec-review.md` on `programs/frontend/` after api contracts are defined
6. On PASS: move frontend to `specced`, begin build

---

## Completion Gate

Project moves to `complete` when:
- All programs have status `complete`
- All contracts in `shared/contracts/` are fulfilled
- No open blocking gaps in `_meta/gaps/CONTEXT.md`
