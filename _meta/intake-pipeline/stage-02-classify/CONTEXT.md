# Stage 02 — Classify

## What This Stage Does
Determines what kind of document this is and how confident that determination is.
Classification drives routing in Stage 03 — no downstream stage should need to re-read
the raw document to make routing decisions.

## Inputs
`_intake/processing/[doc-id]-envelope.json` with `stage: "enveloped"`

## Document Types

| Type | Description | Confidence signals |
|------|-------------|-------------------|
| `prd` | Full product spec — defines programs, contracts, requirements | Contains "programs", "contracts", "requirements"; substantial length (>500 chars) |
| `feature-request` | Addition to an existing project — scoped, not full spec | References existing project name; shorter than a PRD; no program list |
| `adr` | Architecture decision record | Contains "## Decision", "## Status", "## Context"; clearly one decision |
| `contract-update` | Proposed change to a shared contract | References `shared/contracts/`; defines a data shape |
| `question` | Human question or chat message — not a document | No structural headings; interrogative phrasing |
| `unknown` | Cannot be classified with confidence ≥ 0.7 | Ambiguous signals; multiple conflicting type indicators |

## Process

1. **Read `raw_content` from the envelope.**

2. **Score each type.**
   For each document type, count matching signals and compute a confidence score (0.0–1.0).
   Use this heuristic:
   - Count how many signals for type X are present.
   - Divide by total signals for type X.
   - Weight: structural signals (headings, section names) count double.

3. **Select winning type.**
   - Winning type = highest scoring type with score ≥ 0.7.
   - If no type scores ≥ 0.7: type = `unknown`, confidence = highest score.
   - If two types tie: use the more specific type (prd > feature-request > question).

4. **Extract project hint (if possible).**
   Scan for project name signals:
   - Explicit "Project: [name]" or "# [Name] PRD"
   - References to existing `programs/[name]/` folders
   - First significant noun phrase after a title heading
   Store as `project_hint` (may be null if unclear).

5. **Update envelope.**
   Write back to `_intake/processing/[doc-id]-envelope.json`:
   ```json
   {
     ...existing fields...,
     "stage": "classified",
     "classification": {
       "type": "[prd|feature-request|adr|contract-update|question|unknown]",
       "confidence": [0.0-1.0],
       "project_hint": "[slug or null]",
       "signals_matched": ["list of matched signals"]
     }
   }
   ```

6. **Handle `unknown` or low-confidence.**
   If type is `unknown` or confidence < 0.7:
   Log to root `_meta/gaps/pending.txt`:
   ```
   [timestamp] | root | intake-pipeline/stage-02-classify | inferred "document [doc-id] could not be classified with confidence ≥ 0.7 — type signals: [list]" — requires human review
   ```
   Continue to Stage 03 — Stage 03 will route unknowns to the gap system.

## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 3 | type + confidence + signals matched | approve / override type / provide project hint |

Human override is only needed when confidence < 0.9. If confidence ≥ 0.9, skip checkpoint and proceed.

## Audit
- [ ] `classification.type` is one of the seven valid types
- [ ] `classification.confidence` is a float between 0.0 and 1.0
- [ ] `signals_matched` list is non-empty (at least one signal was found)
- [ ] Envelope `stage` updated to `"classified"`

## Outputs
`_intake/processing/[doc-id]-envelope.json` with `stage: "classified"` and `classification` object populated.
