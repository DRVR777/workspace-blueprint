# Gap Registry — project-beta (scope: project-beta)

## What This Is
Project-internal gaps only. Gaps that cross project boundaries go to root `_meta/gaps/`.

## Current Open Gaps

| Gap ID | Type | Location | Description | Severity | Status |
|--------|------|----------|-------------|----------|--------|
| — | — | — | No gaps logged yet | — | — |

## How to Use

1. Agent logs inference to `pending.txt` during any task in this project
2. Run gap detection agent with `scope: project-beta`
3. Gap objects created here for internal gaps
4. Cross-project gaps escalated to root `_meta/gaps/` automatically

## Escalation Rule
If a gap involves any folder outside `project-beta/` — it is a root-scope gap.
Write it to root `_meta/gaps/pending.txt`, not here.
