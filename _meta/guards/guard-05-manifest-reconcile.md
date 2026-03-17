# Guard 5 — MANIFEST Reconciliation Agent

**Source:** ALWS §6
**When to run:** At session end (after validate_manifests.py), or on demand when folder contents may have changed.

---

## What MANIFEST Staleness Means

A MANIFEST.md is stale when its "What I Contain" table no longer matches the actual contents of the folder.

Staleness types:

| Type | Description | Severity |
|------|-------------|----------|
| Missing entry | A file or folder exists in the folder but is not listed in the MANIFEST | high |
| Ghost entry | A file or folder is listed in the MANIFEST but no longer exists | high |
| Wrong purpose | A file's purpose description is clearly wrong (e.g., describes a deleted file's function) | medium |
| Status mismatch | A project's `status` field (scaffold/specced/active/complete) no longer matches reality | high |

---

## Reconciliation Protocol

### Step 1 — Collect
Find all `MANIFEST.md` files in the workspace (exclude `_examples/`, `leftOffHere/`).

### Step 2 — Check each MANIFEST
For each MANIFEST:
1. List actual folder contents (files and immediate subfolders only — not recursive)
2. Compare against "What I Contain" table rows
3. Classify any mismatches using the staleness types above

### Step 3 — Auto-fix low-risk mismatches
Auto-fix **only** these cases without human input:
- New file was added this session and is not yet in MANIFEST → add a row with a placeholder purpose
- Folder was created this session (new program or sub-folder) → add a row

Do NOT auto-fix:
- Ghost entries (the file may have been intentionally deleted — needs human confirmation)
- Status mismatches (status transitions require human sign-off per `_meta/status-transitions.md`)
- Purpose description that looks wrong (agent may be misreading the file)

### Step 4 — Log remaining mismatches
For each mismatch not auto-fixed, write one entry to `_meta/gaps/pending.txt`:

```
[timestamp] | [manifest_path] | guard-05 | inferred "MANIFEST stale: [staleness_type] — [file_or_folder] [description]"
```

### Step 5 — Report
Output a summary:
```
Guard 5 run complete.
MANIFESTs checked: N
Auto-fixed: N rows added
Logged to pending.txt: N entries
```

---

## Integration with validate_manifests.py

`validate_manifests.py` checks for **missing** MANIFESTs (folders with no MANIFEST file at all).
Guard 5 checks for **stale** MANIFESTs (MANIFEST exists but content is wrong).

Run order at session end:
1. `python _meta/scripts/validate_manifests.py` → catches missing MANIFESTs
2. Guard 5 → catches stale MANIFESTs

Both must pass before session closes.

---

## Audit

- [ ] Every MANIFEST.md in scope was checked (count matches)
- [ ] No ghost entries were auto-removed (only additions are auto-applied)
- [ ] All logged gaps use the correct `guard-05` source identifier
- [ ] Summary was output before closing
