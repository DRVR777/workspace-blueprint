# Contract: context-file

Status: defined
Produced By: context-builder
Consumed By: AI (reads directly via file-selector or any agent)

---

## Shape

File path: `Data/ctx-[NNNN].md`

```markdown
# Context — file[NNNN]

Generated: [ISO-8601 timestamp]
Vector: [v1, v2, v3, v4, v5]

## What I Am
[1-3 paragraph description of what this document is about]

## My Position
Vector: [v1, v2, v3, v4, v5] — interpreted as:
- Specificity: [low | medium | high] — [one phrase]
- Technicality: [low | medium | high] — [one phrase]
- Temporality: [stable | mixed | current] — [one phrase]
- Centrality: [peripheral | connected | hub] — [one phrase]
- Confidence: [speculative | probable | established] — [one phrase]

## My Neighbors and How I Relate to Them
| Neighbor | Relationship |
|----------|-------------|
| file[MMMM] | [one sentence: how this file relates to file MMMM] |
| file[MMMM] | ... |

## My Cluster
[single label, e.g. "architectural decisions", "implementation specs", "open questions"]

## My Role
"This document is a [noun] that [verb phrase]."
```

## Rules

- If vector contains nulls: omit My Position section entirely
- If neighbors list is empty: write "No neighbors indexed yet" in the table
- My Role sentence must be a complete sentence in double quotes
- Generated timestamp: current time when context-builder writes the file
- All five sections must be present (My Position excepted if null vector)
- File is overwritten (not appended) on each context-builder run
