# Stage 01 — Envelope

## What This Stage Does
Wraps an inbound document with a stable identity before anything else touches it.
Every downstream stage reads the envelope — not the raw document.

## Inputs
The raw document. One of:
- Inline text (from `intake: "..."` trigger)
- File path (from `_intake/queue/`)
- Direct session content (pasted by human)

## Process

1. **Assign document ID.**
   Generate: `doc-[YYYYMMDD]-[6-char-hash]`
   Hash = first 6 chars of SHA-256 of the document content.
   If hashing is not available: use `doc-[YYYYMMDD]-[random 6-char alphanumeric]`.

2. **Capture source metadata.**
   Record:
   - `source_type`: one of `inline_trigger | queue_file | session_paste | file_path`
   - `source_path`: file path if from queue or file, otherwise `null`
   - `received_at`: ISO-8601 timestamp (UTC)
   - `char_count`: length of raw document in characters
   - `first_line`: first non-empty line of the document (for quick identification)

3. **Write envelope object.**
   Create `_intake/processing/[doc-id]-envelope.json`:
   ```json
   {
     "doc_id": "[doc-id]",
     "received_at": "[ISO-8601]",
     "source_type": "[inline_trigger|queue_file|session_paste|file_path]",
     "source_path": "[path or null]",
     "char_count": [n],
     "first_line": "[first non-empty line]",
     "raw_content": "[full document text]",
     "stage": "enveloped",
     "classification": null,
     "route": null,
     "placed_at": null
   }
   ```

4. **Create processing directory if needed.**
   `_intake/processing/` must exist. Create if absent.

## Checkpoints
| After step | Agent presents | Human options |
|------------|----------------|---------------|
| Step 3 | doc-id assigned, envelope written | approve / reassign id |

## Audit
- [ ] `doc_id` field is unique (no existing envelope with same id)
- [ ] `received_at` is valid ISO-8601
- [ ] `raw_content` is non-empty
- [ ] Envelope file written to `_intake/processing/`

## Outputs
`_intake/processing/[doc-id]-envelope.json` with `stage: "enveloped"`

## Handoff to Stage 02
Pass the envelope file path to Stage 02.
Stage 02 reads `raw_content` from the envelope — it never reads the original source again.
