# Study 04 — Embedded Prompt Effectiveness

Status: designed
Priority: HIGH — validates the self-description layer

---

## Research Question

Does reading a context file (ctx-NNNN.md) actually improve AI navigation
compared to reading only the raw document?

The embedded prompt creates ctx files. If ctx files don't help, the whole
context-builder program is unnecessary overhead.

---

## Hypothesis

**H1:** AI with ctx files reaches its navigation goal in fewer file reads than AI without ctx files.
**H2:** AI with ctx files makes fewer irrelevant reads (less noise).
**H3:** The most valuable section of the ctx file is "My Neighbors and How I Relate to Them"
— this is the section that enables smarter hop decisions.

---

## Test Protocol

### Setup
Two conditions on the same 20-file test set:
- **Condition A (no ctx)**: AI navigates using only raw files. ctx files do not exist.
- **Condition B (with ctx)**: AI navigates with ctx files available. AI reads ctx file immediately after raw file.

### Navigation task (same for both conditions)
Goal: "Find all documents that describe how data flows between programs in the knowledge-graph system."
Expected answer: files 0002, 0010, 0011, 0012, 0007 (architecture + contracts + file-selector)

Run 3 sessions per condition (to average out variance in AI behavior).

### Metrics

| Metric | Condition A | Condition B | Target |
|--------|------------|------------|--------|
| Reads to complete goal | — | — | B < A |
| % of reads that were relevant | — | — | B > A |
| Files missed (not found) | — | — | B ≤ A |
| Dead-end navigations | — | — | B < A |

### Section ablation test
Test Condition C: AI has ctx files but ONLY the "My Neighbors" section is included.
Compare C vs B to determine if the other sections add value.

---

## Embedded Prompt Variation Test

Test 3 different embedded prompt templates:

| Template | What it does |
|----------|-------------|
| Standard | Full prompt: position + neighbors + cluster + role |
| Minimal | Neighbors only: just the relationship list |
| Narrative | No table — paragraph describing position and relationships |

Navigation quality for each? Which template produces the most useful ctx files?
The winner becomes the default template in data-store.

---

## Qualitative Analysis

After each navigation session, read the AI's reasoning (if available in session transcript).
Does the AI cite the ctx file in its reasoning?
Does it use the role sentence or cluster label to make decisions?

Log specific quotes from AI behavior as evidence.

---

## Success Criteria

Condition B shows >15% reduction in reads-to-goal AND >10% improvement in relevance %
→ ctx files are worth building. context-builder stays in the architecture.

Condition B shows <5% improvement on both metrics
→ ctx files are a nice-to-have, not essential. context-builder is lower priority.

Condition B makes navigation WORSE (more reads, less relevant)
→ Embedded prompt template is wrong. Redesign before building context-builder.
