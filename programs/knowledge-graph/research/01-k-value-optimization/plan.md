# Study 01 — K-Value Optimization

Status: designed
ADR resolved: ADR-004
Priority: CRITICAL — blocks indexer implementation

---

## Research Question

How many nearest neighbors should each file list by default?
k=5 was assumed. This study determines whether 5 is the right number.

---

## Why This Matters

k determines:
- How many files the AI reads when following a neighbor chain (branching factor)
- How much context the AI builds per document before navigating away
- The quality of the emergent graph (too few k = disconnected; too many = noisy)
- The performance of the context-builder (more neighbors = more file reads = slower)

Too low: the AI gets stuck in isolated clusters, misses connections
Too high: the AI is overwhelmed with loosely-related content, context files become generic

---

## Hypothesis

**H1:** k=5 produces better navigation quality than k=3 (more context without overwhelm)
**H2:** k=5 produces comparable or better quality than k=10 (10 adds noise, not signal)
**H3:** Optimal k is inversely related to document specificity — abstract hub documents benefit from higher k; specific leaf documents benefit from lower k

---

## Test Protocol

### Setup
Create 3 identical document sets (20 files each) with the same content.
Index each set with a different k value: k=3, k=5, k=10.

Document set: mix of
- 5 hub documents (broad, many connections expected)
- 10 mid-tier documents (moderate specificity)
- 5 leaf documents (highly specific, few connections expected)

### Procedure (per k value)

**Step 1: Navigation quality test**
Give AI a goal: "Find all documents related to [topic X] in this document set."
Let the AI navigate using file-selector. Count:
- Files visited to find all relevant docs
- Files visited that were NOT relevant (noise)
- Dead ends (neighbor chain leads nowhere useful)

**Step 2: Context file quality test**
After navigation, read 5 ctx files (one per document type).
Score each ctx file on:
- Neighbor relationship accuracy (are the described relationships correct?) 1-5
- Cluster label accuracy (does the cluster label make sense?) 1-5
- Role sentence quality (is the role sentence meaningful?) 1-5

**Step 3: Performance measurement**
Count total file-selector calls required to build context for all 20 files.
Record: total time (simulated), total reads, reads per file.

### Configurations tested

| Config | k | Expected strength | Expected weakness |
|--------|---|------------------|------------------|
| A | 3 | Fast, focused | May miss connections |
| B | 5 | Balanced | Baseline |
| C | 10 | Rich context | Noisy, slow |
| D | adaptive | Hub=8, mid=5, leaf=3 | Complex to implement |

Config D (adaptive k) is the stretch goal. Test it only if B vs C is inconclusive.

---

## Success Criteria

Study is concluded when:
- All 4 configurations run on the 20-file test set
- Navigation quality scores computed for each
- A clear winner emerges (difference > 0.5 on 1-5 scale) OR
- No meaningful difference found (within 0.3) → default to k=5 for simplicity

---

## Test Files to Create

See `test-cases/` folder for the 20-document test set.
Documents are about the knowledge-graph system itself (dogfooding).

Topics:
- file0001-0005: architecture and system design (hub level)
- file0006-0015: individual component specs (mid level)
- file0016-0020: specific implementation details (leaf level)

---

## Measurement Template

Fill in after running each configuration:

```
Config [A/B/C/D] — k=[value]
Navigation quality:     [1-5]
Ctx file accuracy:      [1-5]
Ctx file cluster label: [1-5]
Ctx file role sentence: [1-5]
Total file reads:       [n]
Noise reads (irrelevant): [n]
Dead ends:              [n]
```

---

## Expected Conclusion

Based on prior art in k-NN systems and the specific structure of this document system:
k=5 is likely correct for a mixed document set.
However, the adaptive approach (lower k for specific docs) may prove superior for leaf nodes.

If adaptive k is validated: update ADR-004 to describe adaptive behavior.
If k=5 wins: confirm the assumption.
If k=3 wins: simplify the system (fewer reads = faster context building).
