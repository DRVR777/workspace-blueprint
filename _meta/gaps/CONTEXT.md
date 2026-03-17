# Gap Registry

## What This Is

Open gaps in this workspace's knowledge structure. Each gap is a JSON file
in this folder. A gap is something the system doesn't know yet — a missing
file, a missing connection, or a concept not developed to actionable depth.

## Current Open Gaps

| Gap ID | Type | detected_in | Description | Severity | Status |
|--------|------|-------------|-------------|----------|--------|
| gap-001 | missing_bridge | production/CONTEXT.md | production → community handoff undocumented on sender side | degrading | open |
| gap-002 | missing_bridge | writing-room/docs/voice.md | voice.md consumed by community but writing-room doesn't know | degrading | open |
| gap-005 | shallow_node | writing-room/docs/style-guide.md | no content frame → structure mapping (best practices, comparison, explainer, etc.) | degrading | closed |
| gap-003 | shallow_node | production/workflows/01-04 | Stage folders had no MANIFEST | blocking | closed |
| gap-004 | missing_composition | _meta/ | No Layer 0 existed | blocking | closed |

## How Gaps Become Tasks

1. An agent logs an inference to `pending.txt`
2. Gap detection agent classifies it (run `_meta/gap-detection-agent.md`)
3. A gap JSON object is created here
4. The `proposed_expansion` field becomes the task description
5. Feed that description to `_meta/ur-prompt.md` Steps 1–7
6. The output is the execution prompt for the next session

## Gap Detection Trigger

Run gap detection when:
- A task produced output that required agent inference
- A cross-workspace handoff produced unexpected results
- An agent navigated upward to find context it should have had locally
