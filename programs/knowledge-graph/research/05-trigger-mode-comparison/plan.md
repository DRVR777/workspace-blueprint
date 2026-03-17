# Study 05 — Trigger Mode Comparison (On-Read vs Batch)

Status: designed
ADR resolved: ADR-005
Priority: CRITICAL — blocks context-builder implementation

---

## Research Question

Should context-builder run immediately every time a file is read (on-read),
or should it run periodically across all recently-read files (batch)?

---

## Why This Matters

The trigger mode affects:
- **Freshness**: Is the context file up-to-date when the AI reads it?
- **Performance**: How many context-builder runs happen per session?
- **Loop risk**: Can context-builder trigger itself recursively via the ticker?
- **UX**: Does the AI wait for context before navigating, or navigate first?

On-read is better for single-file depth (AI reads file → immediate context).
Batch is better for session-level breadth (AI reads 10 files → batch builds all 10 contexts at once).

---

## Hypothesis

**H1:** On-read produces higher-quality context files because context is built
while the AI still has the document in its active context window.

**H2:** On-read is slower per session but produces better navigation outcomes
(AI makes fewer redundant reads because context files guide it).

**H3:** Batch mode with a short interval (30s) is functionally equivalent to on-read
with lower performance cost — this would make batch the correct default.

---

## Test Protocol

### Configuration A — On-Read
context-builder watches ticker.log. On each new `direct_read` or `neighbor_of_*` entry:
immediately reads the file's neighbors and writes ctx file.

### Configuration B — Batch (30-second intervals)
context-builder runs every 30 seconds. Processes all new ticker entries since last run.
Groups neighbor reads by session — reads each neighbor once even if needed by multiple files.

### Configuration C — Batch (session-end only)
context-builder runs once when the session closes.
Provides the freshest possible batch data but zero help during the session.

### Configuration D — Hybrid
On-read for hub documents (centrality > 0.7).
Batch for mid/leaf documents.
Rationale: hubs are navigated to more often, so their context files are accessed more.

---

## Navigation Quality Test (same for all configs)

**Session goal:** "Navigate the test set and find all documents about contracts and data shapes."

Expected path: start at 0001 or 0002 → find 0010, 0011, 0012 → find related specs.

Measure:
1. **Reads to goal**: how many file-selector calls before all contract docs found
2. **Context availability**: what % of visited files had a ctx file available before the AI needed it
3. **Redundant reads**: how many files were visited more than once in the same session
4. **Context quality**: score the ctx files written (same rubric as Study 01)

---

## Loop Risk Test

Test whether on-read creates infinite loops:

1. context-builder writes ctx-0042.md
2. context-builder appends `context_built` to ticker
3. Does context-builder re-trigger on the `context_built` entry?

Expected: no. The filter `skip if reason == 'context_built'` should prevent loops.
Test this explicitly. Log whether any loop is observed in any configuration.

---

## Performance Test

Run each configuration on the 20-file test set with a scripted "AI" that reads
every file once in a fixed order.

Measure:
- Total context-builder invocations
- Total time for all context files to be written
- Peak memory usage (how many files open simultaneously in batch mode)

---

## Success Criteria

Study is concluded when:
- All 4 configurations run the navigation test
- Loop risk test complete (pass/fail)
- Clear winner on the tradeoff matrix:

| Criterion | Weight | A (on-read) | B (batch 30s) | C (session-end) | D (hybrid) |
|-----------|--------|-------------|--------------|-----------------|-----------|
| Context freshness | 3x | — | — | — | — |
| Navigation quality | 3x | — | — | — | — |
| Performance | 1x | — | — | — | — |
| Loop safety | 2x | — | — | — | — |

Fill in scores 1-5. Weighted sum determines winner.

---

## Staleness Threshold Sub-Study

Regardless of which mode wins, determine the correct staleness threshold:
- If ctx file is younger than T minutes: skip re-running
- Test T = 2, 5, 15, 60 minutes
- Find the T where re-running adds no quality improvement

This applies to on-read mode primarily but informs all modes.

---

## Expected Conclusion

On-read with staleness check (T=5) is the most likely winner.
Batch 30s may tie if the navigation test shows no significant difference in context freshness.
Hybrid is the theoretical optimum but adds implementation complexity — only adopt if A vs B is close.
