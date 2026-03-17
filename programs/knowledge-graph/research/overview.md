# Research Overview — Cognitive Document System

## What This Research Is For

The knowledge-graph program has 4 assumption ADRs that block building.
It also has deeper questions that determine whether the system actually works once built.

This folder turns those questions into runnable experiments with falsifiable hypotheses.
Every study follows the same structure: question → hypothesis → test → findings → decision.

Research is not just for validation. It is for learning what this system actually is
before committing to an architecture that might need to be rebuilt.

---

## The 10 Studies

### Tier 1 — ADR Blockers (must complete before any code is written)

| # | Study | Unblocks | Priority |
|---|-------|---------|---------|
| 01 | K-value optimization | ADR-004 | CRITICAL |
| 05 | Trigger mode comparison | ADR-005 | CRITICAL |
| 06 | Tool vs MCP | ADR-008 | CRITICAL |

These three studies unblock the four assumption ADRs. No program should be built
until each of these has a `status: concluded` finding.

ADR-007 (no deletion) is also an assumption but can be resolved by a simple policy
decision — it does not require empirical research. See `adr-resolution-log.md`.

---

### Tier 2 — System Quality (run during or immediately after initial build)

| # | Study | Tests | Priority |
|---|-------|-------|---------|
| 02 | Vector heuristic accuracy | Do heuristic vectors produce meaningful proximity? | HIGH |
| 03 | Neighbor relevance | Are k-nearest neighbors actually related in content? | HIGH |
| 04 | Embedded prompt effectiveness | Does the prompt produce useful context files? | HIGH |

These studies tell us if the core mechanisms work. Run them with 10-20 test files.
Results feed back into the indexer and context-builder implementations.

---

### Tier 3 — Emergent Behavior (run after the system has accumulated usage)

| # | Study | Tests | Priority |
|---|-------|-------|---------|
| 07 | Ticker as emergent graph | Does navigation history reveal document relationships? | MEDIUM |
| 09 | Context file utility | How much does the AI use ctx files? | MEDIUM |

These require the system to have been used for multiple sessions before results are valid.
Run after 5+ sessions with 20+ files.

---

### Tier 4 — Scale and Architecture (run when Tier 1-2 are complete)

| # | Study | Tests | Priority |
|---|-------|-------|---------|
| 08 | Scalability | Performance at 10/100/1000 files | MEDIUM |
| 10 | 5D vs high-dim | Is 5D as useful as 1536D for navigation? | LOW |

Study 10 is the long-game question. It may not be answerable until the system
has been running for weeks. It also determines whether the ML upgrade path is worth pursuing.

---

## Research Execution Order

```
Week 1: Tier 1 studies (01, 05, 06)
  → All three reach 'concluded' status
  → Four assumption ADRs updated to 'accepted'
  → Build can start

Week 2: Build data-store + file-selector + first Tier 2 studies
  → Studies 02, 03 run in parallel with building indexer
  → Study 04 runs when context-builder is partially built

Week 3+: Tier 3 studies emerge from normal usage
  → Studies 07, 09 track naturally as the system is used

Week 4+: Tier 4 studies when system has 100+ files
  → Studies 08, 10 test limits and validate architecture choices
```

---

## How Studies Work

Each study folder contains:
```
[study]/
  plan.md          ← hypothesis, methodology, test protocol, success criteria
  test-cases/      ← specific inputs: test files, prompts, configurations
  findings/        ← outputs: raw results, analysis, conclusion
```

A study moves through these states:
```
designed → running → concluded
```

A study is `concluded` when:
- The hypothesis is confirmed OR refuted with enough evidence
- A clear recommendation can be written (the ADR can be updated)
- The recommendation is written to `findings/conclusion.md`

---

## What Counts as Evidence

For each study type:

**Configuration studies (k, trigger mode, tool type):**
- Minimum 3 different configurations tested
- Minimum 5 test cases per configuration
- Comparison metric defined before testing (not after)

**Quality studies (vector accuracy, neighbor relevance, prompt effectiveness):**
- Minimum 10 test documents
- Human evaluation of output quality (1-5 scale)
- Quantitative metric where possible (e.g., % neighbors that are thematically related)

**Emergent studies (ticker, context utility):**
- Minimum 5 sessions with at least 10 file reads per session
- Requires real usage (cannot be simulated)

---

## Connection to the Rest of the System

Research findings that change the architecture → update the corresponding ADR.
Findings that reveal new unknowns → new gap in `_meta/gaps/pending.txt`.
Findings that suggest new research questions → new study folder here.

This folder is a living document. Studies add rows, findings update status.
Nothing here is ever deleted — only superseded.
