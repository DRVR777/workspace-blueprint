# Study 02 — Vector Heuristic Accuracy

Status: designed
ADR resolved: None (quality study — informs indexer implementation)
Priority: HIGH — run while building indexer

---

## Research Question

Do heuristic rules (vocabulary analysis, language signals) produce 5D vectors
that meaningfully represent a document's semantic position?

Or do they produce random-looking numbers that happen to be in [0.0, 1.0]?

---

## Hypothesis

**H1:** Heuristic vectors will correctly rank documents on Specificity and Technicality
because these dimensions have clear vocabulary signals (proper nouns, code blocks, technical terms).

**H2:** Heuristic vectors will be less accurate for Centrality because centrality
depends on usage patterns (ticker data) not just content — and ticker data doesn't exist yet.

**H3:** Vectors computed from heuristics will produce neighbor lists with >60% overlap
with ground-truth neighbor lists (from test-set-design.md) — enough to be useful.

---

## Ground Truth

Use the expected vectors and expected neighbor relationships from:
`01-k-value-optimization/test-cases/test-set-design.md`

Ground truth vectors were manually assigned based on deep understanding of each document.
These are the "correct answers" the heuristic is trying to match.

---

## Test Protocol

### Step 1: Implement the heuristic indexer (minimal version)
Write `indexer/src/heuristic.py` — compute all 5 dimensions for a given document text.
(This is a research prototype — not production code.)

### Step 2: Run on the 20-file test set
For each file, compute the heuristic vector.
Record in findings/raw-vectors.md:
```
file0001: heuristic=[0.x, 0.x, 0.x, 0.x, 0.x] | expected=[0.3, 0.3, 0.1, 1.0, 0.9]
```

### Step 3: Score accuracy per dimension

For each dimension, compute Mean Absolute Error (MAE):
```
MAE_dim1 = mean(|heuristic[dim1] - expected[dim1]|) across all 20 files
```

Target: MAE < 0.2 for each dimension.
(A MAE of 0.2 means the heuristic is off by an average of 0.2 on a 0-1 scale — acceptable.)

### Step 4: Score neighbor accuracy

For each file, compare heuristic-computed top-5 neighbors vs ground-truth top-5.
Metric: Jaccard similarity = |intersection| / |union|

Target: mean Jaccard > 0.5 (more than half the neighbors match).

### Step 5: Identify worst-performing dimension

Which dimension has highest MAE? That dimension needs a better heuristic or manual calibration.
Document the specific failure cases.

---

## Scoring Rubric

| Outcome | Interpretation | Action |
|---------|---------------|--------|
| MAE < 0.15 all dims | Heuristic is accurate | Proceed with heuristic implementation |
| MAE < 0.2, Jaccard > 0.5 | Heuristic is good enough | Proceed, note known limitations |
| MAE 0.2-0.3, Jaccard 0.3-0.5 | Heuristic is partially useful | Proceed but flag ML upgrade as higher priority |
| MAE > 0.3 or Jaccard < 0.3 | Heuristic is not useful | Redesign heuristics or switch to ML embeddings |

---

## Sub-study: Calibration

If the heuristic produces systematically biased vectors (all Specificity scores too high, etc.),
test linear calibration: multiply each dimension output by a constant factor.

Find calibration constants that minimize MAE.
Document them — they become hardcoded multipliers in the indexer implementation.

---

## Connection to Study 10

Study 10 compares 5D vectors to high-dimensional embeddings.
Study 02 is a prerequisite: if heuristics are too inaccurate, the Study 10 comparison
is between "bad heuristic" vs "good ML" which is an unfair test.
Complete Study 02 and fix the heuristics before running Study 10.
