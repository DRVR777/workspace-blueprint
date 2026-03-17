# Gap Registry — project-alpha

## Scope: project-alpha-internal only
Cross-project gaps go to `{root}/_meta/gaps/`.

## Current Open Gaps

| Gap ID | Type | detected_in | Description | Severity | Status |
|--------|------|-------------|-------------|----------|--------|
| — | — | — | No gaps yet — populate from PRD intake | — | — |

## Resolution Protocol

**Blocking gaps (PRD unknowns / assumption ADRs):**
Do not write code in the affected program until closed.
Close by: writing an accepted ADR in `_planning/adr/`, updating gap status here.

**Degrading/cosmetic gaps:**
Follow standard session protocol in `{root}/_meta/runner.md`.

## Escalation Rule
If a gap involves anything outside programs/project-alpha/ → `{root}/_meta/gaps/pending.txt` instead.
