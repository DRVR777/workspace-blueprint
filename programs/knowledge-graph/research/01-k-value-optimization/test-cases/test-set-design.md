# Test Set Design — Study 01

20 documents about the knowledge-graph system itself.
Chosen because: we already know the true relationships, which makes scoring easy.

---

## Document Map (expected relationships)

```
HUB LAYER (files 0001-0005) — each should connect to most other files
  0001: System overview — what CDS is, why it exists
  0002: Architecture overview — 4 programs, data flow
  0003: 5D vector concept — why 5 interpretable dims
  0004: Navigation philosophy — why tool-use not passive retrieval
  0005: Ticker/attention trace — what the log becomes over time

MID LAYER (files 0006-0015) — connect to 1-2 hubs + each other
  0006: data-store spec — file creation, naming
  0007: file-selector spec — read, log, return
  0008: indexer spec — heuristic computation
  0009: context-builder spec — ticker watching, ctx writing
  0010: file-record contract — shared data shape
  0011: ticker-entry contract — log line shape
  0012: context-file contract — ctx shape
  0013: embedded prompt template — what fires on read
  0014: k-nearest neighbor algorithm — distance formula
  0015: 5D distance and weighting — proximity query

LEAF LAYER (files 0016-0020) — connect to 1 mid, few others
  0016: file naming convention detail — NNNN padding rules
  0017: ticker rotation policy — when log gets large
  0018: access_count increment logic — edge case: concurrent reads
  0019: staleness threshold config — context-builder skip logic
  0020: metadata parser implementation — line-by-line reader
```

## Ground Truth Relationships

For scoring neighbor relevance in Study 03, expected neighbors:

| File | Expected top-5 neighbors |
|------|-------------------------|
| 0001 | 0002, 0004, 0005, 0003, 0009 |
| 0002 | 0001, 0006, 0007, 0008, 0009 |
| 0003 | 0001, 0014, 0015, 0008, 0004 |
| 0004 | 0001, 0007, 0005, 0009, 0013 |
| 0005 | 0004, 0001, 0007, 0011, 0009 |
| 0006 | 0002, 0010, 0016, 0007, 0020 |
| 0007 | 0002, 0010, 0011, 0005, 0009 |
| 0008 | 0002, 0014, 0015, 0003, 0010 |
| 0009 | 0002, 0012, 0013, 0004, 0005 |
| 0010 | 0006, 0007, 0008, 0009, 0011 |
| 0011 | 0007, 0005, 0010, 0012, 0009 |
| 0012 | 0009, 0013, 0010, 0011, 0004 |
| 0013 | 0004, 0009, 0012, 0001, 0007 |
| 0014 | 0008, 0015, 0003, 0010, 0006 |
| 0015 | 0014, 0003, 0008, 0007, 0004 |
| 0016 | 0006, 0020, 0010, 0017, 0007 |
| 0017 | 0005, 0011, 0016, 0019, 0020 |
| 0018 | 0007, 0016, 0006, 0011, 0020 |
| 0019 | 0009, 0013, 0017, 0012, 0018 |
| 0020 | 0006, 0016, 0010, 0018, 0008 |

## Expected 5D Vectors (ground truth for Study 02)

| File | Specificity | Technicality | Temporality | Centrality | Confidence | Label |
|------|------------|-------------|-------------|-----------|-----------|-------|
| 0001 | 0.3 | 0.3 | 0.1 | 1.0 | 0.9 | hub/overview |
| 0002 | 0.4 | 0.5 | 0.1 | 0.9 | 0.9 | hub/architecture |
| 0003 | 0.4 | 0.4 | 0.1 | 0.8 | 0.9 | hub/concept |
| 0004 | 0.3 | 0.3 | 0.1 | 0.8 | 0.9 | hub/philosophy |
| 0005 | 0.4 | 0.4 | 0.2 | 0.8 | 0.8 | hub/behavior |
| 0006 | 0.7 | 0.7 | 0.2 | 0.5 | 0.8 | mid/spec |
| 0007 | 0.7 | 0.7 | 0.2 | 0.6 | 0.8 | mid/spec |
| 0008 | 0.7 | 0.8 | 0.2 | 0.5 | 0.7 | mid/spec |
| 0009 | 0.7 | 0.6 | 0.2 | 0.5 | 0.7 | mid/spec |
| 0010 | 0.8 | 0.8 | 0.2 | 0.6 | 0.9 | mid/contract |
| 0011 | 0.8 | 0.7 | 0.2 | 0.5 | 0.9 | mid/contract |
| 0012 | 0.8 | 0.7 | 0.2 | 0.4 | 0.9 | mid/contract |
| 0013 | 0.7 | 0.5 | 0.2 | 0.5 | 0.8 | mid/template |
| 0014 | 0.9 | 0.9 | 0.1 | 0.4 | 0.9 | mid/algorithm |
| 0015 | 0.9 | 0.9 | 0.1 | 0.4 | 0.9 | mid/algorithm |
| 0016 | 1.0 | 0.8 | 0.1 | 0.2 | 1.0 | leaf/detail |
| 0017 | 1.0 | 0.6 | 0.3 | 0.1 | 0.6 | leaf/policy |
| 0018 | 1.0 | 0.9 | 0.2 | 0.1 | 0.6 | leaf/edge-case |
| 0019 | 1.0 | 0.7 | 0.2 | 0.1 | 0.6 | leaf/config |
| 0020 | 1.0 | 0.9 | 0.1 | 0.2 | 0.8 | leaf/implementation |

These are the "correct answers" — used to score heuristic vector accuracy in Study 02.
