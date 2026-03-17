# Study 09 — Context File Utility

Status: designed
Priority: MEDIUM — requires real sessions to observe AI behavior

---

## Research Question

When ctx files exist alongside raw files, does the AI actually use them?
And if so, which sections does it use to make navigation decisions?

This is an observability study — we watch what the AI does, not what it says it does.

---

## Hypothesis

**H1:** The AI will read ctx files in most sessions when they exist, because the
ctx file is listed as a resource in the file metadata (`context_file:` field).

**H2:** The "My Neighbors" section will be the most-cited section in AI navigation reasoning.

**H3:** The AI will sometimes skip ctx files and navigate directly — this happens
when the raw file content is rich enough to make the next navigation decision.

---

## Observability Protocol

After each session, review the session transcript (if available) and ticker.log.

**From ticker.log:** Count ctx file reads (file-selector calls to ctx-NNNN.md files)
vs raw file reads (file-selector calls to fileNNNN.md files).
Ratio: ctx_reads / raw_reads = "context usage rate"

**From session transcript (if available):** When AI makes a navigation decision,
does it cite the ctx file or the raw file as its source?
Tag each navigation decision: source = [ctx | raw | both | unclear]

---

## Metrics

| Metric | Target | Why |
|--------|--------|-----|
| Context usage rate | > 0.5 | AI reads a ctx file at least half as often as it reads raw files |
| % nav decisions citing ctx | > 30% | ctx files influence at least a third of navigation choices |
| % ctx files that are read (coverage) | > 70% | most visited files have their ctx read |

---

## Failure Modes

**AI never reads ctx files:**
Reason: ctx files are not surfaced to the AI correctly (not in file metadata, not in embedded prompt)
Fix: ensure the embedded prompt says "read my context file at [path] before navigating"

**AI reads ctx but doesn't use them:**
Reason: ctx files are too generic, not useful for navigation
Fix: Study 04 revisions — improve embedded prompt quality

**AI ignores ctx and uses raw files perfectly:**
Reason: ctx files are redundant for short/simple documents
Conclusion: ctx files only add value for complex or hub documents — make them optional
