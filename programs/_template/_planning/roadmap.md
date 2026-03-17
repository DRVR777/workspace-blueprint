# Roadmap — {{PROJECT_NAME}}

## Build Order

Programs must build in dependency order per `{root}/_meta/status-transitions.md`.
A program cannot move to `specced` if any contract it *consumes* is produced by
a program still at `scaffold`.

| Build Order | Program | Depends On | Status |
|-------------|---------|-----------|--------|
| [fill from PRD] | `programs/[name]/` | [none or dependency] | scaffold |

---

## Status

| Program | Status | Blocking | Notes |
|---------|--------|---------|-------|
| [program] | scaffold | [assumption ADRs, stub contracts] | — |

---

## Next Actions

1. Resolve all `assumption` ADRs in `_planning/adr/` — change to `accepted` after human review
2. Define shapes for all stub contracts in `shared/contracts/`
3. Run `{root}/_meta/spec-review.md` on each program in build order above
4. On PASS: move program to `specced`, begin build

---

## Completion Gate

Project moves to `complete` when:
- All programs have status `complete`
- All contracts in `shared/contracts/` are fulfilled
- No open blocking gaps in `_meta/gaps/CONTEXT.md`
