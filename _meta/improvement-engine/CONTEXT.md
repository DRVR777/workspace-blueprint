# Improvement Engine — Agent Contract

## What This Is

The self-modification pipeline. It reads telemetry emitted by every stage run,
finds patterns, and proposes diffs that improve the workspace over time.

It does NOT touch content. It modifies instructions, contracts, and routing rules.

---

## Trigger

Run this when:
- `improve` keyword is typed (immediate run)
- A stage has accumulated 5+ telemetry entries (batch analysis ready)
- A gap of type `shallow_node` is closed (re-evaluate affected stage)

Do NOT run during a build session. Run at session end, after gap detection.

---

## Inputs

| Input | Source | Required |
|-------|--------|----------|
| Telemetry files | `programs/[project]/programs/[prog]/output/telemetry.json` | Yes |
| Stage contracts | `programs/[project]/programs/[prog]/CONTEXT.md` | Yes |
| Diff logs | `programs/[project]/programs/[prog]/output/diff-log.json` | If exists |
| Historical pattern baseline | `_meta/improvement-engine/baseline.json` (auto-created) | No |

---

## Process

### Step 1 — Collect
Scan all `output/telemetry.json` files reachable from the workspace root.
Load each. Group by `stage_id`.

Only include entries where `status: completed`. Skip `status: partial` or `status: failed`
(those become gap objects, not improvement candidates).

### Step 2 — Detect Patterns
For each stage group, look for:

| Pattern | Signal | Threshold |
|---------|--------|-----------|
| Slow step | `duration_ms` > 2× median for that stage | 3 occurrences |
| Audit failure | `audit_flags` contains `FAIL` items | 2 occurrences |
| Input not loaded | `inputs_skipped` contains a file listed in CONTEXT.md's What to Load | 3 occurrences |
| Output missing | Expected output file absent from `outputs_produced` | 2 occurrences |
| Route mismatch | `routing_confidence` < 0.7 in intake-pipeline telemetry | 3 occurrences |

When a pattern threshold is crossed → create a diff proposal (Step 3).

### Step 3 — Propose Diffs
For each detected pattern, write a diff entry to the affected stage's `diff-log.json`.

Classify the diff:

| Type | Criteria | Action |
|------|----------|--------|
| `low-risk` | Changes a threshold, reorders a checklist item, adds a missing `What to Load` entry | Auto-apply (Step 4) |
| `high-impact` | Changes a contract's Process steps, adds/removes an output, modifies routing rules | Human-gate (Step 5) |

Every diff entry requires:
- `evidence`: array of `run_id` values that triggered it
- `description`: plain English — what changes and why
- `before` / `after`: the exact text being replaced

### Step 4 — Auto-Apply Low-Risk Diffs
For each `low-risk` diff:
1. Confirm the `before` text still matches the current file exactly
2. Apply the change
3. Set `status: applied`, add `applied_at` timestamp
4. Log to `_meta/gaps/pending.txt`:
   `[timestamp] | [stage path] | diff-log.json | applied low-risk diff "[description]"`

### Step 5 — Surface High-Impact Diffs
For each `high-impact` diff:
1. Write it to `diff-log.json` with `status: proposed`
2. Add one entry to `_meta/gaps/pending.txt`:
   `[timestamp] | [stage path] | diff-log.json | inferred "high-impact diff requires human review: [description]"`
3. Stop. Do not apply. The gap system routes it to the human next session.

---

## Outputs

| Output | Location | Notes |
|--------|----------|-------|
| Applied diffs | Stage `output/diff-log.json` | status: applied |
| Proposed diffs | Stage `output/diff-log.json` | status: proposed — await human |
| Pattern log entries | `_meta/gaps/pending.txt` | One entry per pattern detected |
| Baseline update | `_meta/improvement-engine/baseline.json` | Written after every run |

---

## Audit

Before closing an improvement run, verify:

- [ ] Every pattern that crossed its threshold has a diff entry
- [ ] Every `low-risk` diff has `evidence` with ≥ 1 valid `run_id`
- [ ] No `high-impact` diff was auto-applied
- [ ] `baseline.json` was updated with this run's median `duration_ms` per stage
- [ ] All `pending.txt` entries use the correct format

If any check fails → do not mark the run complete. Fix and re-verify.

---

## Constraints

- This agent never modifies `_core/CONVENTIONS.md` directly. Pattern → gap → human review → CONVENTIONS update.
- This agent never touches `finalized.flag` files. Finalized artifacts are frozen.
- A diff that would change a stage's Inputs or Outputs contract is always `high-impact`, regardless of scope.
- Maximum 3 auto-applies per run. If more `low-risk` diffs are ready, queue them for the next run.
