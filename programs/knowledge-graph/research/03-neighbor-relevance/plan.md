# Study 03 — Neighbor Relevance

Status: phase-1-complete
Priority: HIGH — validates the core navigation mechanism

---

## Research Question

When an AI reads file X and then reads file X's neighbors — are those neighbors
actually relevant to what the AI is trying to understand?

A neighbor in 5D space is close in the vector sense. But is it useful in the navigation sense?

---

## Hypothesis

**H1:** Neighbors computed from accurate 5D vectors will be thematically related
to the source document >70% of the time.

**H2:** High-centrality documents (hubs) will have more relevant neighbors than
low-centrality leaf documents, because hubs are generally related to many things.

**H3:** After 5+ sessions, ticker-informed centrality will produce better neighbor
lists than initial heuristic centrality — the system improves with use.

---

## Test Protocol

### Phase 1 — Static relevance (before any sessions)
Use the 20-file test set with vectors from Study 02.
For each file, look at its top-5 heuristic neighbors.
Human rater scores each neighbor pair on relevance: 1 (unrelated) to 5 (strongly related).

Metric: mean relevance score across all 100 neighbor pairs (20 files × 5 neighbors).
Target: mean score > 3.5 (relevant more often than not).

### Phase 2 — Navigation-guided relevance
Run a navigation session: AI tries to understand the "contract" subsystem.
After session, look at which neighbor chains the AI followed.
Were the neighbors it followed actually useful for the goal?

Human rater (or agent) scores: for each navigation hop, was the destination useful? Y/N.
Metric: % useful hops.
Target: > 65% useful hops.

### Phase 3 — Post-session improvement (requires Study 07 data)
After 5 sessions, centrality values in the vectors are updated from ticker data.
Re-run Phase 1 with updated vectors.
Does mean relevance score improve?

---

## Relevance Scoring Rubric

For each (source, neighbor) pair:

| Score | Meaning |
|-------|---------|
| 5 | Directly related — reading the neighbor is essential for understanding the source |
| 4 | Closely related — adds useful context |
| 3 | Tangentially related — might be useful |
| 2 | Weakly related — surface similarity, little content overlap |
| 1 | Unrelated — proximity in 5D space was misleading |

---

## Failure Mode Analysis

For every neighbor pair scored 1 or 2: record WHY the 5D distance was low.
Common failure modes:
- **Technicality collision**: two documents with same technicality score but unrelated topics
- **Temporal collision**: two current documents that happen to be in same time period
- **Hub gravity**: all files end up near hubs because hubs have high centrality

If a failure mode appears in >20% of cases: fix the heuristic for that dimension.

---

## Success Criteria

Study is concluded when Phase 1 and 2 are complete.
Phase 3 is ongoing (requires live usage data).

Phase 1 + 2 conclusion: "Neighbor lists are [useful/marginally useful/not useful] for navigation."
This informs whether the system needs ML embeddings before it's worth building.
