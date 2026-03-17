"""
validate_manifests.py — Check every MANIFEST.md for schema compliance.

Usage:
    python validate_manifests.py             # check entire workspace
    python validate_manifests.py --fix       # auto-fix simple issues (depth, parent)
    python validate_manifests.py --missing   # only report folders without MANIFEST.md

What it checks:
1. MISSING — Routing folders that have no MANIFEST.md
2. SCHEMA — Each MANIFEST.md has required fields: id, type, depth, parent, status
3. DEPTH — `depth` field matches actual path depth
4. PARENT — `parent` field matches actual parent path
5. PLACEHOLDERS — Non-template MANIFESTs don't contain {{PLACEHOLDER}} values
6. STALE CONTENTS — Files/folders listed in "What I Contain" that don't exist on disk

Exit code: 0 if all pass, 1 if any failures.

"Routing folders" = any folder an agent would navigate to.
"Leaf folders" = adr/, queue/, processed/, output/ — MANIFEST optional.
"""

import os
import sys
import re
from pathlib import Path

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "..", ".."))

# Leaf folders: MANIFEST is optional here
LEAF_FOLDER_NAMES = {"adr", "queue", "processed", "output", "constants", "src", "tests", "dist"}

# Folders to completely skip (not part of the workspace system)
SKIP_FOLDERS = {".git", ".claude", "__pycache__", "node_modules", ".venv", "venv"}

# Required envelope fields
REQUIRED_FIELDS = ["id", "type", "depth", "parent", "status"]


# ── Path helpers ──────────────────────────────────────────────────────────────

def rel_to_root(abs_path: str) -> str:
    rel = os.path.relpath(abs_path, WORKSPACE_ROOT)
    return rel.replace("\\", "/")


def path_depth(abs_path: str) -> int:
    rel = os.path.relpath(abs_path, WORKSPACE_ROOT)
    if rel == ".":
        return 0
    return len(rel.replace("\\", "/").split("/"))


def is_leaf_folder(folder_name: str) -> bool:
    return folder_name.lower() in LEAF_FOLDER_NAMES


# ── MANIFEST field reader ─────────────────────────────────────────────────────

def read_field(content: str, field: str) -> str | None:
    pattern = rf"\| `{re.escape(field)}` \| (.+?) \|"
    match = re.search(pattern, content)
    if match:
        return match.group(1).strip()
    return None


def read_all_fields(content: str) -> dict[str, str]:
    pattern = r"\| `(\w+)` \| (.+?) \|"
    return {m.group(1): m.group(2).strip() for m in re.finditer(pattern, content)}


# ── "What I Contain" parser ───────────────────────────────────────────────────

def parse_contains_table(content: str) -> list[str]:
    """Return list of names from the 'What I Contain' table."""
    section_match = re.search(r"## What I Contain(.*?)(?=\n## |\Z)", content, re.DOTALL)
    if not section_match:
        return []
    section = section_match.group(1)
    names = []
    for line in section.split("\n"):
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.split("|") if c.strip()]
        if not cells or cells[0] in ("Name", "---", "------", "----"):
            continue
        name = cells[0].rstrip("/")
        if name and name != "[none yet]":
            names.append(cells[0])  # Keep trailing slash if present
    return names


# ── Walker ────────────────────────────────────────────────────────────────────

def walk_routing_folders() -> list[str]:
    """Return all folder paths that should have MANIFEST.md."""
    routing_folders = []
    for dirpath, dirnames, _ in os.walk(WORKSPACE_ROOT):
        # Prune skip folders
        dirnames[:] = [
            d for d in dirnames
            if d not in SKIP_FOLDERS and not d.startswith(".")
        ]
        # Root itself
        if dirpath == WORKSPACE_ROOT:
            routing_folders.append(dirpath)
            continue
        folder_name = os.path.basename(dirpath)
        if not is_leaf_folder(folder_name):
            routing_folders.append(dirpath)
    return routing_folders


# ── Validators ────────────────────────────────────────────────────────────────

class Issue:
    def __init__(self, severity: str, path: str, check: str, message: str, fix: str = ""):
        self.severity = severity  # ERROR | WARNING | INFO
        self.path = path
        self.check = check
        self.message = message
        self.fix = fix  # If non-empty: a suggested fix string

    def __str__(self) -> str:
        parts = [f"  [{self.severity}] {self.check}: {self.path}"]
        parts.append(f"    {self.message}")
        if self.fix:
            parts.append(f"    Fix: {self.fix}")
        return "\n".join(parts)


def check_missing(folder: str) -> Issue | None:
    manifest = os.path.join(folder, "MANIFEST.md")
    if not os.path.exists(manifest):
        rel = rel_to_root(folder)
        return Issue(
            "ERROR", rel, "MISSING",
            "No MANIFEST.md found.",
            f"python _meta/scripts/scaffold_manifest.py {rel} --update-parent"
        )
    return None


def check_schema(folder: str, content: str) -> list[Issue]:
    issues = []
    rel = rel_to_root(folder)
    fields = read_all_fields(content)
    for field in REQUIRED_FIELDS:
        if field not in fields:
            issues.append(Issue(
                "ERROR", rel, "SCHEMA",
                f"Missing required envelope field: `{field}`",
                f"Add `| `{field}` | [value] |` to the Envelope table"
            ))
    return issues


def check_depth(folder: str, content: str, fix: bool) -> Issue | None:
    rel = rel_to_root(folder)
    actual_depth = path_depth(folder)
    declared = read_field(content, "depth")
    if declared is None:
        return None  # Caught by schema check
    try:
        declared_int = int(declared)
    except ValueError:
        return Issue("ERROR", rel, "DEPTH", f"depth field is not an integer: '{declared}'")
    if declared_int != actual_depth:
        issue = Issue(
            "WARNING", rel, "DEPTH",
            f"depth is {declared} but actual depth is {actual_depth}.",
            f"Change `| `depth` | {declared} |` to `| `depth` | {actual_depth} |`"
        )
        if fix:
            manifest_path = os.path.join(folder, "MANIFEST.md")
            new_content = re.sub(
                rf"(\| `depth` \| ){re.escape(declared)}( \|)",
                rf"\g<1>{actual_depth}\g<2>",
                content,
            )
            if new_content != content:
                with open(manifest_path, "w", encoding="utf-8") as f:
                    f.write(new_content)
                print(f"  AUTO-FIXED depth: {rel} → {actual_depth}")
                return None
        return issue
    return None


def check_parent(folder: str, content: str, fix: bool) -> Issue | None:
    rel = rel_to_root(folder)
    actual_parent = rel_to_root(os.path.dirname(folder))
    declared = read_field(content, "parent")
    if declared is None:
        return None
    # Normalize: strip trailing slash for comparison
    declared_norm = declared.rstrip("/").rstrip("\\")
    actual_norm = actual_parent.rstrip("/")
    if declared_norm != actual_norm and actual_norm != ".":
        issue = Issue(
            "WARNING", rel, "PARENT",
            f"parent is '{declared}' but actual parent is '{actual_parent}/'.",
            f"Change parent to `{actual_parent}/`"
        )
        if fix:
            manifest_path = os.path.join(folder, "MANIFEST.md")
            new_content = re.sub(
                rf"(\| `parent` \| ).+?( \|)",
                rf"\g<1>{actual_parent}/\g<2>",
                content,
            )
            if new_content != content:
                with open(manifest_path, "w", encoding="utf-8") as f:
                    f.write(new_content)
                print(f"  AUTO-FIXED parent: {rel} → {actual_parent}/")
                return None
        return issue
    return None


def check_placeholders(folder: str, content: str) -> Issue | None:
    rel = rel_to_root(folder)
    # Skip the template itself
    if "_template" in folder.replace("\\", "/"):
        return None
    if "{{" in content and "}}" in content:
        placeholders = re.findall(r"\{\{[A-Z_]+\}\}", content)
        return Issue(
            "WARNING", rel, "PLACEHOLDERS",
            f"Contains unresolved placeholders: {', '.join(set(placeholders))}",
            "Replace placeholders with actual values, or verify this is intentionally a template."
        )
    return None


def check_stale_contents(folder: str, content: str) -> list[Issue]:
    issues = []
    rel = rel_to_root(folder)
    names = parse_contains_table(content)
    for name in names:
        # name may have trailing slash (folder) or not (file)
        clean = name.rstrip("/")
        candidate = os.path.join(folder, clean)
        if not os.path.exists(candidate):
            issues.append(Issue(
                "WARNING", rel, "STALE-CONTENTS",
                f"'What I Contain' lists '{name}' but it doesn't exist on disk.",
                f"Either create {clean} or remove the row from the table."
            ))
    return issues


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    args = sys.argv[1:]
    fix_mode = "--fix" in args
    missing_only = "--missing" in args

    print(f"\n{'═'*60}")
    print("  MANIFEST VALIDATION")
    print(f"  Root: {WORKSPACE_ROOT}")
    if fix_mode:
        print("  Mode: AUTO-FIX (depth + parent fields)")
    print(f"{'═'*60}\n")

    folders = walk_routing_folders()
    all_issues: list[Issue] = []
    checked = 0
    missing_count = 0

    for folder in sorted(folders):
        manifest_path = os.path.join(folder, "MANIFEST.md")

        # Check 1: Missing
        missing_issue = check_missing(folder)
        if missing_issue:
            all_issues.append(missing_issue)
            missing_count += 1
            if missing_only:
                print(missing_issue)
            continue  # Can't check schema if file doesn't exist

        if missing_only:
            continue  # Only want missing report

        with open(manifest_path, "r", encoding="utf-8") as f:
            content = f.read()

        checked += 1

        # Check 2: Schema
        all_issues.extend(check_schema(folder, content))

        # Check 3: Depth
        depth_issue = check_depth(folder, content, fix_mode)
        if depth_issue:
            all_issues.append(depth_issue)

        # Re-read after potential fix
        with open(manifest_path, "r", encoding="utf-8") as f:
            content = f.read()

        # Check 4: Parent
        parent_issue = check_parent(folder, content, fix_mode)
        if parent_issue:
            all_issues.append(parent_issue)

        # Re-read after potential fix
        with open(manifest_path, "r", encoding="utf-8") as f:
            content = f.read()

        # Check 5: Placeholders
        ph_issue = check_placeholders(folder, content)
        if ph_issue:
            all_issues.append(ph_issue)

        # Check 6: Stale contents
        all_issues.extend(check_stale_contents(folder, content))

    # Report
    errors = [i for i in all_issues if i.severity == "ERROR"]
    warnings = [i for i in all_issues if i.severity == "WARNING"]

    if not missing_only:
        if all_issues:
            for issue in all_issues:
                print(issue)
                print()
        else:
            print("  All MANIFESTs valid.")

    print(f"{'─'*60}")
    print(f"  Folders checked: {checked}")
    print(f"  Missing MANIFESTs: {missing_count}")
    print(f"  Errors: {len(errors)}")
    print(f"  Warnings: {len(warnings)}")

    if missing_count > 0:
        print()
        print("  To generate missing stubs:")
        print("    python _meta/scripts/scaffold_manifest.py <folder-path> --update-parent")

    if not fix_mode and (errors or warnings):
        print()
        print("  Run with --fix to auto-correct depth and parent fields.")

    print()
    sys.exit(1 if errors else 0)


if __name__ == "__main__":
    main()
