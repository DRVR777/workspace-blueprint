"""
run_gaps.py — Parse pending.txt and print gap candidates for agent review.

Usage:
    python run_gaps.py                    # root scope
    python run_gaps.py --scope <project>  # project scope
    python run_gaps.py --all              # all scopes

What it does:
1. Reads the appropriate pending.txt file(s)
2. Prints unprocessed entries grouped by scope
3. Optionally shows a summary of open gaps from CONTEXT.md

This script surfaces the work for an agent to do — it does not generate
gap JSON files itself (that's the job of _meta/gap-detection-agent.md).
"""

import os
import sys
import re
from datetime import datetime

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIR))


def read_pending(path: str) -> list[str]:
    """Read non-comment, non-empty lines from a pending.txt."""
    if not os.path.exists(path):
        return []
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    return [
        line.rstrip()
        for line in lines
        if line.strip() and not line.strip().startswith("#")
    ]


def find_all_pending_files() -> dict[str, str]:
    """Return {scope_name: path} for all pending.txt files in the workspace."""
    result = {}
    root_pending = os.path.join(WORKSPACE_ROOT, "_meta", "gaps", "pending.txt")
    if os.path.exists(root_pending):
        result["root"] = root_pending

    programs_dir = os.path.join(WORKSPACE_ROOT, "programs")
    if os.path.exists(programs_dir):
        for project in os.listdir(programs_dir):
            if project.startswith("_"):
                continue
            project_pending = os.path.join(
                programs_dir, project, "_meta", "gaps", "pending.txt"
            )
            if os.path.exists(project_pending):
                result[project] = project_pending
    return result


def print_entries(scope: str, entries: list[str]) -> None:
    if not entries:
        print(f"  [scope: {scope}] No entries.")
        return
    print(f"\n  [scope: {scope}] {len(entries)} entries:")
    for entry in entries:
        print(f"    {entry}")


def main() -> None:
    args = sys.argv[1:]
    all_files = find_all_pending_files()

    if "--all" in args:
        scopes = all_files
    elif "--scope" in args:
        idx = args.index("--scope") + 1
        if idx >= len(args):
            print("ERROR: --scope requires a value")
            sys.exit(1)
        scope_name = args[idx]
        if scope_name not in all_files:
            print(f"ERROR: Scope '{scope_name}' not found. Available: {list(all_files.keys())}")
            sys.exit(1)
        scopes = {scope_name: all_files[scope_name]}
    else:
        # Default: root scope
        scopes = {"root": all_files.get("root", "")} if "root" in all_files else {}

    print("=== PENDING INFERENCE LOG ===")
    if not scopes:
        print("No pending.txt files found.")
        sys.exit(0)

    total = 0
    for scope, path in scopes.items():
        entries = read_pending(path)
        total += len(entries)
        print_entries(scope, entries)

    print(f"\nTotal: {total} entries across {len(scopes)} scope(s)")
    print()
    print("Next: Run _meta/gap-detection-agent.md to classify these into formal gap objects.")


if __name__ == "__main__":
    main()
