# Guard 3 — finalized.flag Cascade Protection

**Source:** ALWS §15
**When to run:** Before writing any file to an `output/` folder or any folder that may contain finalized artifacts.

---

## What finalized.flag Means

A `finalized.flag` file in a folder means:
- Every artifact in that folder is frozen
- No file in that folder may be modified or deleted
- New files may not be added to that folder
- The flag itself may not be deleted

`finalized.flag` cascades **downward**: if a parent folder is finalized, all child folders are also finalized.

`finalized.flag` does **not** cascade upward: a finalized subfolder does not finalize its parent.

---

## Guard Check Protocol

**Before writing any file, run these checks in order:**

### Check 1 — Direct flag
Does the target folder contain `finalized.flag`?
- Yes → **BLOCK**. Do not write. Log to `_meta/gaps/pending.txt`:
  `[timestamp] | [target_folder] | guard-03 | BLOCKED write to finalized folder: [file_path]`

### Check 2 — Ancestor flag
Does any parent folder of the target (up to workspace root) contain `finalized.flag`?
- Yes → **BLOCK**. Same log format as Check 1, note the ancestor folder.

### Check 3 — Clear
No `finalized.flag` found in target or any ancestor → proceed with write.

---

## How to Finalize a Folder

Only a human may create a `finalized.flag` file. Agents never create this flag.

To finalize:
1. Human creates an empty file named `finalized.flag` in the target folder
2. Human logs the finalization to `_meta/gaps/pending.txt`:
   `[timestamp] | [folder_path] | human | finalized folder: [reason]`
3. The folder is now frozen — Guard 3 will block any agent write attempts

---

## Exceptions

None. There are no exceptions to finalized.flag protection.
If a finalized artifact needs to be changed, the human must:
1. Delete `finalized.flag` manually
2. Log the unfinalization with reason
3. Make the change
4. Re-create `finalized.flag` if appropriate

---

## Audit (run at session end)

- [ ] Scan workspace for all `finalized.flag` files — list them in pending.txt if any were created this session
- [ ] Verify no agent-written file is younger than any `finalized.flag` in its ancestor path
- [ ] If a violation is found: log as `critical` gap, do not attempt to reverse the write without human instruction
