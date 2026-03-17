# Study 07 — Ticker as Emergent Graph

Status: designed
Priority: MEDIUM — requires live usage data (5+ sessions)

---

## Research Question

After multiple sessions of real use, does ticker.log contain enough information
to materialize a meaningful graph of document relationships?

The claim: you never need to define relationships between documents manually.
They emerge from navigation patterns.

---

## Hypothesis

**H1:** Documents co-accessed within the same session (within N reads of each other)
will be more semantically related than random document pairs.

**H2:** The graph materialized from ticker.log will have meaningful clusters —
groups of documents that are consistently navigated together.

**H3:** The ticker-derived centrality values will match human-judged importance
better than heuristic centrality after 10+ sessions.

---

## Metrics

### Co-access edge weight
For files A and B: edge weight = number of sessions where A and B were both read
within a window of 5 reads of each other.

High edge weight → probably related.
Low/zero edge weight → probably not related or not yet discovered.

### Cluster detection
Apply simple community detection to the co-access graph.
Do the clusters align with the expected topic groups (hub/mid/leaf from test-set-design.md)?

### Centrality vs human judgment
After 10 sessions: rank files by ticker access count.
Compare to human ranking of "most important files to understand the system."
Spearman correlation — target > 0.7.

---

## Data Collection Protocol

After each session using the knowledge-graph system:
1. Run `python research/07-ticker-as-emergent-graph/scripts/analyze_ticker.py`
2. Output: co-access matrix, top 10 edges, detected clusters
3. Save to `findings/session-[N]-analysis.md`

After 5 sessions: run comparative analysis.
After 10 sessions: run full study.

---

## The Interesting Finding to Watch For

**Unexpected connections**: files that weren't expected to be related (from test-set-design.md
ground truth) but appear as high-weight edges in the ticker graph.

These are discoveries. The system found a relationship that the human designer didn't.
Log all unexpected connections as potential new knowledge.
