# Stage 05 — Update

## What This Stage Does
Closes the intake loop. Archives the envelope, logs routing confidence,
moves source file to processed/, and records the intake event.
After this stage: the document is fully integrated and the pipeline is clean.

## Inputs
`_intake/processing/[doc-id]-envelope.json` with `stage: "placed"`

## Process

1. **Archive the envelope.**
   Move (copy + delete) `_intake/processing/[doc-id]-envelope.json`
   to `_intake/processed/[doc-id]-envelope.json`.

2. **Move source file to processed/ (if from queue).**
   If `source_type == "queue_file"` and `source_path` exists:
   Move the source file from `_intake/queue/[filename]`
   to `_intake/processed/[slug]-[ISO-date].[ext]`.
   Format: `[project-hint]-prd-[YYYY-MM-DD].[ext]` for PRDs.

3. **Append to intake log.**
   Append one line to `_intake/intake-log.md` (create if absent):
   ```
   | [ISO-date] | [doc-id] | [type] | [confidence] | [handler] | [target_project] | [n files placed] |
   ```
   The log file format:
   ```markdown
   # Intake Log

   | Date | Doc ID | Type | Confidence | Handler | Project | Files Placed |
   |------|--------|------|------------|---------|---------|-------------|
   ```

4. **Log routing confidence to pending.txt.**
   If `classification.confidence < 0.85`:
   ```
   [timestamp] | root | intake-pipeline/stage-05-update | inferred "document [doc-id] was classified as [type] with low confidence [n] — routing may need review" — no file states this
   ```
   This surfaces low-confidence classifications for human review without blocking.

5. **Log intake inference to root pending.txt.**
   Always append:
   ```
   [timestamp] | root | intake-pipeline/stage-05-update | inferred "document [doc-id] processed: type=[type], placed=[n files], project=[target_project]" — intake pipeline
   ```

6. **Print completion summary.**
   ```
   INTAKE COMPLETE
   ===============
   Document ID:  [doc-id]
   Type:         [type] (confidence: [n])
   Handler:      [handler]
   Project:      [target_project or "new"]
   Files placed: [n]
   Archived:     _intake/processed/[doc-id]-envelope.json

   [If confidence < 0.85:]
   NOTE: Low classification confidence. Check _meta/gaps/pending.txt for review flag.
   ```

## Checkpoints
No checkpoints — Stage 05 is fully autonomous. If any step fails, log to pending.txt and stop.

## Audit
- [ ] Envelope moved from `processing/` to `processed/` (no longer in processing/)
- [ ] Source file moved from `queue/` (if applicable)
- [ ] Intake log updated with correct row
- [ ] Root pending.txt appended (at minimum the intake inference entry)
- [ ] `_intake/processing/` is now empty (or contains only in-flight envelopes)

## Outputs
- `_intake/processed/[doc-id]-envelope.json`
- `_intake/intake-log.md` (updated)
- Root `_meta/gaps/pending.txt` (appended)
