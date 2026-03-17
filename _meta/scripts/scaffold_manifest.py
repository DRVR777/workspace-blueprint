"""
scaffold_manifest.py — Generate a MANIFEST.md stub for any folder.

Usage:
    python scaffold_manifest.py <folder-path>
    python scaffold_manifest.py <folder-path> --type project
    python scaffold_manifest.py <folder-path> --update-parent

What it does:
1. Takes a folder path (relative to workspace root or absolute)
2. Infers: depth, parent, id, type from path structure
3. Writes a MANIFEST.md stub into that folder
4. Optionally appends a row to the parent MANIFEST.md "What I Contain" table

Run this immediately after creating any new folder.
Convention: no folder is considered complete until it has MANIFEST.md.

Type inference rules (override with --type if wrong):
    _meta/         → meta
    _planning/     → planning
    _core/         → conventions
    _intake/       → intake
    setup/         → onboarding
    programs/      → programs-container  (if depth 1)
                   → programs            (if depth 3+)
    shared/        → contracts
    gaps/          → gaps
    scripts/       → scripts
    output/        → output  (leaf — MANIFEST usually not needed, but supported)
    Any folder named after a project in programs/ at depth 2 → project
    Any folder named after a program inside programs/*/programs/ at depth 4 → program
    Everything else → folder
"""

import os
import sys
import re
from datetime import datetime, timezone

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "..", ".."))


# ── Path helpers ──────────────────────────────────────────────────────────────

def resolve_path(raw: str) -> str:
    """Return absolute path. Accepts workspace-root-relative or absolute."""
    if os.path.isabs(raw):
        return os.path.normpath(raw)
    return os.path.normpath(os.path.join(WORKSPACE_ROOT, raw))


def rel_to_root(abs_path: str) -> str:
    """Return path relative to workspace root, using forward slashes."""
    rel = os.path.relpath(abs_path, WORKSPACE_ROOT)
    return rel.replace("\\", "/")


def path_depth(abs_path: str) -> int:
    """
    Depth relative to workspace root.
    Root itself = 0. Direct children = 1. etc.
    """
    rel = os.path.relpath(abs_path, WORKSPACE_ROOT)
    if rel == ".":
        return 0
    parts = rel.replace("\\", "/").split("/")
    return len(parts)


def path_parts(abs_path: str) -> list[str]:
    rel = os.path.relpath(abs_path, WORKSPACE_ROOT)
    return rel.replace("\\", "/").split("/")


# ── Type inference ─────────────────────────────────────────────────────────────

FOLDER_NAME_TO_TYPE = {
    "_meta": "meta",
    "_planning": "planning",
    "_core": "conventions",
    "_intake": "intake",
    "_examples": "reference",
    "_registry": "registry",
    "setup": "onboarding",
    "shared": "contracts",
    "gaps": "gaps",
    "scripts": "scripts",
    "output": "output",
    "adr": "adr",
    "queue": "queue",
    "processed": "processed",
    "constants": "constants",
}


def infer_type(abs_path: str) -> str:
    parts = path_parts(abs_path)
    name = parts[-1]
    depth = len(parts)

    # Exact name matches
    if name in FOLDER_NAME_TO_TYPE:
        return FOLDER_NAME_TO_TYPE[name]

    # programs/ is context-dependent
    if name == "programs":
        if depth == 1:
            return "programs-container"
        return "programs"

    # depth 1 → top-level workspace folder
    if depth == 1:
        return "workspace-folder"

    # depth 2 inside programs/ → project
    if depth == 2 and len(parts) >= 2 and parts[0] == "programs":
        return "project"

    # depth 4 inside programs/*/programs/* → program
    if depth == 4 and len(parts) == 4:
        if parts[0] == "programs" and parts[2] == "programs":
            return "program"

    return "folder"


# ── ID generation ─────────────────────────────────────────────────────────────

def make_id(abs_path: str) -> str:
    """Generate a unique id from the path: parts joined with hyphens."""
    parts = path_parts(abs_path)
    # Strip leading underscores from folder names for cleaner IDs
    cleaned = [p.lstrip("_") for p in parts]
    return "-".join(cleaned)


# ── Status inference ───────────────────────────────────────────────────────────

def infer_status(folder_type: str) -> str:
    if folder_type == "output":
        return "empty"
    if folder_type in ("queue", "processed"):
        return "active"
    return "scaffold"


# ── MANIFEST stub generation ───────────────────────────────────────────────────

def scan_contents(abs_path: str) -> list[tuple[str, str, str]]:
    """
    Return [(name, type, purpose)] for direct children of abs_path.
    purpose is always a placeholder — human/agent fills it in.
    """
    contents = []
    try:
        entries = sorted(os.listdir(abs_path))
    except PermissionError:
        return contents
    for entry in entries:
        entry_path = os.path.join(abs_path, entry)
        if entry == "MANIFEST.md":
            continue  # Don't list the MANIFEST itself
        if entry.startswith("."):
            continue
        if os.path.isdir(entry_path):
            contents.append((entry + "/", "folder", "[purpose — fill in]"))
        else:
            contents.append((entry, "file", "[purpose — fill in]"))
    return contents


def generate_manifest(abs_path: str, folder_type: str) -> str:
    rel = rel_to_root(abs_path)
    depth = path_depth(abs_path)
    parent_rel = rel_to_root(os.path.dirname(abs_path)) if depth > 0 else "none"
    folder_id = make_id(abs_path)
    status = infer_status(folder_type)
    folder_name = os.path.basename(abs_path)
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    contents = scan_contents(abs_path)
    if contents:
        contents_rows = "\n".join(
            f"| {name} | {t} | {purpose} |"
            for name, t, purpose in contents
        )
    else:
        contents_rows = "| [none yet] | — | — |"

    return f"""# MANIFEST — {rel}/

## Envelope
| Field | Value |
|-------|-------|
| `id` | {folder_id} |
| `type` | {folder_type} |
| `depth` | {depth} |
| `parent` | {parent_rel}/ |
| `status` | {status} |
| `created` | {timestamp} |

## What I Am
[Describe what this folder is and does — 1-2 sentences]

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
{contents_rows}

## Routing Rules
| Condition | Go To |
|-----------|-------|
| [task] | [file or folder] |
"""


# ── Parent MANIFEST update ─────────────────────────────────────────────────────

def update_parent_manifest(abs_path: str) -> bool:
    """
    Append a row for abs_path to parent MANIFEST.md "What I Contain" table.
    Skips if the entry already exists. Returns True if updated.
    """
    parent_dir = os.path.dirname(abs_path)
    parent_manifest = os.path.join(parent_dir, "MANIFEST.md")
    if not os.path.exists(parent_manifest):
        print(f"  WARNING: Parent MANIFEST not found at {rel_to_root(parent_manifest)}")
        print("  Run scaffold_manifest.py on the parent folder first.")
        return False

    with open(parent_manifest, "r", encoding="utf-8") as f:
        content = f.read()

    folder_name = os.path.basename(abs_path)
    entry_name = folder_name + "/"

    # Already present?
    if f"| {entry_name} |" in content or f"| {folder_name} |" in content:
        print(f"  Parent MANIFEST already has entry for '{folder_name}' — skipping.")
        return False

    # Find the last row of "What I Contain" table and append after it
    # Pattern: find "## What I Contain" section, then the last | row before the next ##
    contains_match = re.search(
        r"(## What I Contain.*?)(\n## |\Z)",
        content,
        re.DOTALL,
    )
    if not contains_match:
        print("  WARNING: Could not find 'What I Contain' table in parent MANIFEST.")
        print("  Add this row manually:")
        print(f"  | {entry_name} | folder | [purpose] |")
        return False

    section = contains_match.group(1)
    # Find the last table row in the section
    rows = [line for line in section.split("\n") if line.startswith("|")]
    if not rows:
        print("  WARNING: No table rows found in 'What I Contain'. Add manually.")
        return False

    last_row = rows[-1]
    new_row = f"| {entry_name} | folder | [purpose — fill in] |"
    updated_content = content.replace(last_row, last_row + "\n" + new_row, 1)

    with open(parent_manifest, "w", encoding="utf-8") as f:
        f.write(updated_content)

    print(f"  Updated {rel_to_root(parent_manifest)} — added row for '{folder_name}/'")
    return True


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    args = sys.argv[1:]
    if not args or args[0] in ("-h", "--help"):
        print(__doc__)
        sys.exit(0)

    raw_path = args[0]
    update_parent = "--update-parent" in args

    # Override type
    folder_type = None
    if "--type" in args:
        idx = args.index("--type") + 1
        if idx < len(args):
            folder_type = args[idx]

    abs_path = resolve_path(raw_path)

    if not os.path.isdir(abs_path):
        print(f"ERROR: Not a directory: {abs_path}")
        print("Create the folder first, then run this script.")
        sys.exit(1)

    manifest_path = os.path.join(abs_path, "MANIFEST.md")
    if os.path.exists(manifest_path):
        print(f"MANIFEST.md already exists at {rel_to_root(manifest_path)}")
        print("Delete it first if you want to regenerate.")
        sys.exit(0)

    if folder_type is None:
        folder_type = infer_type(abs_path)
        print(f"Inferred type: '{folder_type}' (override with --type <type>)")

    manifest_content = generate_manifest(abs_path, folder_type)

    with open(manifest_path, "w", encoding="utf-8") as f:
        f.write(manifest_content)

    rel = rel_to_root(abs_path)
    print(f"Created MANIFEST.md at {rel}/MANIFEST.md")

    if update_parent:
        update_parent_manifest(abs_path)
    else:
        print()
        print("Tip: run with --update-parent to also add this folder to the parent MANIFEST.md")

    print()
    print("Next: open the MANIFEST and fill in:")
    print("  - 'What I Am' description")
    print("  - Purpose column in 'What I Contain'")
    print("  - Routing Rules rows")


if __name__ == "__main__":
    main()
