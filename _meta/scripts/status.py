"""
status.py — Show what has been produced vs what is still empty.

Implements the `status` trigger keyword from root CLAUDE.md.

Usage:
    python status.py                     # full workspace status
    python status.py --project <name>    # one project only
    python status.py --summary           # counts only, no file lists

What it reports:
1. For every project in programs/: overall project status (from MANIFEST.md)
2. For every program in each project: program status + output/ contents
3. Open gaps summary (count from pending.txt files)
4. Overall completion percentage
"""

import os
import sys
import re
from datetime import datetime, timezone

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "..", ".."))
PROGRAMS_PATH = os.path.join(WORKSPACE_ROOT, "programs")


# ── MANIFEST readers ──────────────────────────────────────────────────────────

def read_manifest_field(manifest_path: str, field: str) -> str | None:
    """Extract a value from the Envelope table in a MANIFEST.md."""
    if not os.path.exists(manifest_path):
        return None
    with open(manifest_path, "r", encoding="utf-8") as f:
        content = f.read()
    pattern = rf"\| `{re.escape(field)}` \| (.+?) \|"
    match = re.search(pattern, content)
    if match:
        return match.group(1).strip()
    return None


def read_status(manifest_path: str) -> str:
    return read_manifest_field(manifest_path, "status") or "unknown"


# ── Output folder scanner ─────────────────────────────────────────────────────

def scan_output(output_path: str) -> list[str]:
    """Return list of files in an output/ folder (excluding .gitkeep)."""
    if not os.path.isdir(output_path):
        return []
    files = [
        f for f in os.listdir(output_path)
        if not f.startswith(".") and f != ".gitkeep"
        and os.path.isfile(os.path.join(output_path, f))
    ]
    return sorted(files)


# ── Gap counter ───────────────────────────────────────────────────────────────

def count_pending(scope_path: str) -> int:
    """Count non-comment, non-empty lines in a pending.txt."""
    pending = os.path.join(scope_path, "_meta", "gaps", "pending.txt")
    if not os.path.exists(pending):
        return 0
    with open(pending, "r", encoding="utf-8") as f:
        lines = f.readlines()
    return sum(
        1 for line in lines
        if line.strip() and not line.strip().startswith("#")
    )


def count_open_gaps(scope_path: str) -> int:
    """Count open gap JSON files in a project's _meta/gaps/."""
    gaps_dir = os.path.join(scope_path, "_meta", "gaps")
    if not os.path.isdir(gaps_dir):
        return 0
    count = 0
    for fname in os.listdir(gaps_dir):
        if not fname.endswith(".json"):
            continue
        fpath = os.path.join(gaps_dir, fname)
        try:
            with open(fpath, "r", encoding="utf-8") as f:
                content = f.read()
            if '"status": "open"' in content:
                count += 1
        except (OSError, UnicodeDecodeError):
            pass
    return count


# ── Project/program discovery ─────────────────────────────────────────────────

def get_projects() -> list[str]:
    """Return sorted list of project folder names (excluding _template)."""
    if not os.path.isdir(PROGRAMS_PATH):
        return []
    return sorted(
        d for d in os.listdir(PROGRAMS_PATH)
        if os.path.isdir(os.path.join(PROGRAMS_PATH, d))
        and not d.startswith("_")
        and d != "_template"
    )


def get_programs(project_path: str) -> list[str]:
    """Return sorted list of program folder names inside a project's programs/."""
    programs_dir = os.path.join(project_path, "programs")
    if not os.path.isdir(programs_dir):
        return []
    return sorted(
        d for d in os.listdir(programs_dir)
        if os.path.isdir(os.path.join(programs_dir, d))
        and not d.startswith("_")
        and not d.startswith(".")
    )


# ── Status symbols ────────────────────────────────────────────────────────────

STATUS_SYMBOL = {
    "complete": "✓",
    "active": "▶",
    "specced": "◆",
    "scaffold": "○",
    "template": "⊡",
    "unknown": "?",
    "empty": "·",
}


def sym(status: str) -> str:
    return STATUS_SYMBOL.get(status.lower(), "?")


# ── Report builder ────────────────────────────────────────────────────────────

def report_project(project_name: str, summary_only: bool = False) -> dict:
    """
    Print status for one project. Return stats dict.
    """
    project_path = os.path.join(PROGRAMS_PATH, project_name)
    manifest_path = os.path.join(project_path, "MANIFEST.md")
    status = read_status(manifest_path)
    pending = count_pending(project_path)
    open_gaps = count_open_gaps(project_path)
    programs = get_programs(project_path)

    program_stats = []
    for prog_name in programs:
        prog_path = os.path.join(project_path, "programs", prog_name)
        prog_manifest = os.path.join(prog_path, "MANIFEST.md")
        prog_status = read_status(prog_manifest)
        output_files = scan_output(os.path.join(prog_path, "output"))
        program_stats.append({
            "name": prog_name,
            "status": prog_status,
            "output_files": output_files,
        })

    # Print
    prog_count = len(programs)
    complete_progs = sum(1 for p in program_stats if p["status"] == "complete")
    print(f"\n{'─'*60}")
    print(f"  {sym(status)} {project_name}  [{status}]  "
          f"programs: {complete_progs}/{prog_count} complete  "
          f"pending: {pending}  open-gaps: {open_gaps}")

    if not summary_only:
        for p in program_stats:
            out = p["output_files"]
            out_str = f"  output: {', '.join(out)}" if out else "  output: (empty)"
            print(f"      {sym(p['status'])} {p['name']}  [{p['status']}]{out_str}")

    return {
        "status": status,
        "programs_total": prog_count,
        "programs_complete": complete_progs,
        "pending": pending,
        "open_gaps": open_gaps,
    }


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    args = sys.argv[1:]
    summary_only = "--summary" in args
    target_project = None

    if "--project" in args:
        idx = args.index("--project") + 1
        if idx < len(args):
            target_project = args[idx]

    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    print(f"\n{'═'*60}")
    print(f"  WORKSPACE STATUS — {timestamp}")
    print(f"  Root: {WORKSPACE_ROOT}")
    print(f"{'═'*60}")

    # Legend
    print()
    print("  Legend: ✓ complete  ▶ active  ◆ specced  ○ scaffold")

    if target_project:
        projects = [target_project]
    else:
        projects = get_projects()

    if not projects:
        print("\n  No projects found in programs/")
        sys.exit(0)

    totals = {"programs_total": 0, "programs_complete": 0, "pending": 0, "open_gaps": 0}
    status_counts: dict[str, int] = {}

    for project in projects:
        project_path = os.path.join(PROGRAMS_PATH, project)
        if not os.path.isdir(project_path):
            print(f"\n  ERROR: Project not found: {project}")
            continue
        stats = report_project(project, summary_only)
        for key in ("programs_total", "programs_complete", "pending", "open_gaps"):
            totals[key] += stats[key]
        s = stats["status"]
        status_counts[s] = status_counts.get(s, 0) + 1

    # Root pending
    root_pending_path = os.path.join(WORKSPACE_ROOT, "_meta", "gaps", "pending.txt")
    root_pending = 0
    if os.path.exists(root_pending_path):
        with open(root_pending_path, "r", encoding="utf-8") as f:
            root_pending = sum(
                1 for line in f
                if line.strip() and not line.strip().startswith("#")
            )

    print(f"\n{'─'*60}")
    print(f"  TOTALS — {len(projects)} project(s)")
    for status, count in sorted(status_counts.items()):
        print(f"    {sym(status)} {status}: {count}")
    print(f"    Programs: {totals['programs_complete']}/{totals['programs_total']} complete")
    print(f"    Pending inferences: {totals['pending']} project-scope + {root_pending} root-scope")
    print(f"    Open gaps: {totals['open_gaps']}")

    if totals["programs_total"] > 0:
        pct = int(100 * totals["programs_complete"] / totals["programs_total"])
        print(f"    Completion: {pct}%")

    print()
    if totals["pending"] + root_pending > 0:
        print("  → Pending inferences exist. Run `run gaps` to classify them.")
    if totals["open_gaps"] > 0:
        print("  → Open gaps exist. Run `run gaps` to close them.")
    print()


if __name__ == "__main__":
    main()
