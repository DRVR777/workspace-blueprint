# Gap Registry — knowledge-graph

## Scope: knowledge-graph internal only
Cross-project gaps go to root _meta/gaps/.

## Current Open Gaps

| Gap ID | Type | Description | Severity | Status |
|--------|------|-------------|----------|--------|
| gap-001 | missing_composition | k value for neighbors (ADR-004) not validated by human | high | open |
| gap-002 | missing_composition | context-builder trigger mode (ADR-005) not validated | medium | open |
| gap-003 | missing_composition | file-selector tool implementation (ADR-008) not validated | high | open |
| gap-004 | missing_composition | file deletion policy (ADR-007) not validated | low | open |

## Resolution Protocol

All `assumption` ADRs above require human validation before the affected program builds.
1. Open _planning/adr/[NNN]-[slug].md
2. Review the assumption
3. Change `status: assumption` to `status: accepted` (or rewrite the decision)
4. Update gap status here to `closed`
5. Then build the program that was waiting

## Escalation Rule
If a gap involves coordination with other projects → root _meta/gaps/pending.txt.
