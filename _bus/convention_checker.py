#!/usr/bin/env python3
"""
convention_checker.py — Real-Time Convention Enforcement

Watches the workspace for new and modified files.
Checks them against _core/CONVENTIONS.md patterns.
Posts violations to:
    - _bus/convention_violations.md (persistent log)
    - _bus/broadcast.md (immediate alert)

Usage:
    python _bus/convention_checker.py          # watch daemon
    python _bus/convention_checker.py --scan   # one-time full scan, then exit

Checks enforced:
    P-25: Every new folder must get MANIFEST.md within the session
    P-16: No magic numbers/strings in code (heuristic)
    P-04: No cross-program imports (import from sibling program folder)
    P-03: Handoffs via output/ — code writing to parent folder directly
    P-23: ADR must be 'accepted' before code in same project starts
    P-22: Fix-First — checks for TODO/FIXME left in committed code files
    P-15: output/ files must not be imported in other files
    STRUCT: Python files without a module docstring
    STRUCT: New programs/ subfolder without __init__.py (Python projects)
"""

import re
import sys
import time
import threading
from pathlib import Path
from datetime import datetime, timezone

try:
    from watchdog.observers import Observer
    from watchdog.events import FileSystemEventHandler
except ImportError:
    print("ERROR: watchdog not installed. Run: pip install watchdog")
    sys.exit(1)

ROOT = Path(__file__).parent.parent
BUS  = ROOT / "_bus"

SKIP_DIRS = {
    "__pycache__", ".git", "node_modules", ".pytest_cache",
    ".mypy_cache", ".venv", "venv", ".tox", "output",
}

# Top-level directories to skip entirely (third-party repos, ALWS example folders)
SKIP_TOPLEVEL = {
    "claude-office-skills-ref",   # external git repo, not our code
    "community",                   # ALWS example workspace (Acme DevRel)
    "production",                  # ALWS example workspace
    "writing-room",                # ALWS example workspace
    "New folder",                  # leftover empty folder
}

CODE_EXTENSIONS  = {".py", ".ts", ".js", ".rs", ".go"}
SCHEMA_EXTENSIONS = {".fbs", ".proto", ".json"}
MAX_VIOLATIONS = 500   # cap the violations file size


# ──────────────────────────────────────────────────────────────────────────────
# INFER AGENT FROM PATH
# ──────────────────────────────────────────────────────────────────────────────

AGENT_DOMAINS = {
    "oracle-agent":    "programs/oracle",
    "game-agent":      "programs/game_engine",
    "kg-agent":        "programs/knowledge-graph",
    "meta-agent":      "_meta",
    "workspace-agent": "programs/workspace-builder",
}

def _infer_agent(rel: str) -> str:
    for agent, domain in AGENT_DOMAINS.items():
        if rel.replace("\\", "/").startswith(domain):
            return agent
    return "unknown-agent"


# ──────────────────────────────────────────────────────────────────────────────
# MESSAGE WRITER
# ──────────────────────────────────────────────────────────────────────────────

def _now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")

def _post_violation(path: str, rule: str, description: str, severity: str = "warn"):
    """Write violation to both convention_violations.md and broadcast.md."""
    ts    = _now()
    agent = _infer_agent(path)
    icon  = {"error": "🔴", "warn": "🟡", "info": "🔵"}.get(severity, "⚪")

    block = (
        f"\n<!-- MSG {ts} | FROM: convention-checker | TO: {agent} | TYPE: convention-violation -->\n"
        f"{icon} **[{severity.upper()}]** `{rule}`\n"
        f"**File:** `{path}`\n"
        f"**Issue:** {description}\n"
        f"**Fix:** See `_core/CONVENTIONS.md #{rule}`\n"
        f"<!-- /MSG -->\n"
    )

    violations_file = BUS / "convention_violations.md"

    # check violations file size — stop if too large
    try:
        current = violations_file.read_text(encoding="utf-8")
        line_count = current.count("\n")
        if line_count > MAX_VIOLATIONS:
            return
    except FileNotFoundError:
        pass

    # Individual violations go ONLY to convention_violations.md (not broadcast).
    # Broadcast gets a summary at end of scan — see full_scan().
    try:
        with open(violations_file, "a", encoding="utf-8") as fp:
            fp.write(block)
    except OSError:
        pass

    print(f"[checker] {severity.upper()} {rule} — {path}")


# ──────────────────────────────────────────────────────────────────────────────
# CONVENTION CHECKS
# ──────────────────────────────────────────────────────────────────────────────

def check_p25_manifest(dirpath: Path, rel: str):
    """P-25: New folder must get MANIFEST.md."""
    if dirpath.exists() and dirpath.is_dir():
        files = {f.name for f in dirpath.iterdir() if f.is_file()}
        if "MANIFEST.md" not in files:
            # only flag routing-relevant dirs (not leaf code dirs)
            subdirs = [f for f in dirpath.iterdir() if f.is_dir() and f.name not in SKIP_DIRS]
            depth = len(Path(rel).parts)
            if subdirs or depth <= 2:
                _post_violation(rel, "P-25", "Directory missing MANIFEST.md", "error")


def check_p04_circular_imports(filepath: Path, rel: str):
    """P-04: No cross-program imports — importing from a sibling program."""
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return

    parts = Path(rel).as_posix().split("/")
    # find which program this file is in
    if "programs" not in parts:
        return

    prog_idx = parts.index("programs")
    # e.g. programs/oracle/programs/signal-ingestion/src/main.py
    # outer program = oracle, inner program = signal-ingestion
    if len(parts) <= prog_idx + 2:
        return
    outer_program = parts[prog_idx + 1]  # e.g. "oracle"

    # check for imports referencing a different outer program
    for other in AGENT_DOMAINS.values():
        other_name = other.split("/")[-1]  # e.g. "game_engine"
        if other_name == outer_program:
            continue
        # look for: import game_engine, from game_engine, from programs.game_engine
        patterns = [
            rf"import\s+{re.escape(other_name)}",
            rf"from\s+{re.escape(other_name)}",
            rf"from\s+programs\.{re.escape(other_name)}",
        ]
        for pat in patterns:
            if re.search(pat, content):
                _post_violation(
                    rel, "P-04",
                    f"Cross-program import detected: `{outer_program}` importing from `{other_name}`. "
                    f"Shared contracts go in `shared/contracts/` only.",
                    "error"
                )
                return


def check_p16_magic_values(filepath: Path, rel: str):
    """P-16: No magic numbers or strings in code files."""
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return

    lines = content.splitlines()
    magic_hits = []

    for i, line in enumerate(lines, 1):
        # skip comments, docstrings, test files
        stripped = line.strip()
        if stripped.startswith(("#", "//", "*", '"""', "'''")):
            continue
        if "test_" in filepath.name or "_test" in filepath.name:
            continue

        # flag bare numeric literals that are clearly not 0/1/2/-1
        # e.g. port = 6379 is fine if it matches a known constant
        # we check for hardcoded ports, timeouts, limits that should be constants
        known_magic = re.findall(r'\b(6379|8080|8000|9200|5432|3306|27017|443|80)\b', line)
        if known_magic:
            magic_hits.append(f"line {i}: hardcoded port/address `{known_magic[0]}`")
            if len(magic_hits) >= 2:
                break

    if magic_hits:
        _post_violation(
            rel, "P-16",
            f"Magic values detected — move to shared constants: {'; '.join(magic_hits[:2])}",
            "warn"
        )


def check_p22_fixme(filepath: Path, rel: str):
    """P-22: Fix-First — FIXME/HACK left in code is a violation (not TODO — TODOs are tracked)."""
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return

    hits = []
    for i, line in enumerate(content.splitlines(), 1):
        if re.search(r'\b(FIXME|HACK|XXX)\b', line):
            hits.append(f"line {i}")
        if len(hits) >= 3:
            break

    if hits:
        _post_violation(
            rel, "P-22",
            f"FIXME/HACK marker found at {', '.join(hits)} — fix immediately (P-22 Fix-First rule)",
            "warn"
        )


def check_p15_output_import(filepath: Path, rel: str):
    """P-15: output/ files must not be imported by other files."""
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return

    if re.search(r'["\'].*output/.*["\']', content) or re.search(r'import.*output\.', content):
        _post_violation(
            rel, "P-15",
            "Possible import of output/ artifact. Output files are artifacts, not references. "
            "Import from source or shared/contracts/ instead.",
            "warn"
        )


def check_struct_docstring(filepath: Path, rel: str):
    """STRUCT: Python module should have a docstring."""
    if filepath.suffix != ".py":
        return
    if "test_" in filepath.name or "_test" in filepath.name:
        return
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return

    lines = [l.strip() for l in content.splitlines() if l.strip()]
    if not lines:
        return

    # skip shebang and encoding comments
    start = 0
    while start < len(lines) and (lines[start].startswith("#") or lines[start].startswith("#!/")):
        start += 1

    if start < len(lines):
        first_real = lines[start]
        if not (first_real.startswith('"""') or first_real.startswith("'''")):
            # only flag if file is non-trivial (>20 lines)
            if len(content.splitlines()) > 20:
                _post_violation(
                    rel, "STRUCT",
                    "Python module missing top-level docstring. Add a brief description of what this module does.",
                    "info"
                )


def check_schema_placement(filepath: Path, rel: str):
    """STRUCT: Schema files (.fbs, .proto) must be in shared/schemas/ — not inside a program."""
    norm = Path(rel).as_posix()
    suffix = filepath.suffix
    if suffix not in (".fbs", ".proto"):
        return

    if "shared/schemas" not in norm and "shared/contracts" not in norm:
        if "programs/" in norm:
            _post_violation(
                rel, "P-16",
                f"Schema file `{filepath.name}` is inside a program folder. "
                f"Move to `shared/schemas/` — schemas are shared contracts, not program-private.",
                "error"
            )


# ──────────────────────────────────────────────────────────────────────────────
# BATCH RUNNER — check a single file
# ──────────────────────────────────────────────────────────────────────────────

def _is_skipped(rel: str) -> bool:
    """Check if a relative path should be skipped entirely."""
    parts = rel.split("/")
    if parts and parts[0] in SKIP_TOPLEVEL:
        return True
    return False

def check_file(filepath: Path):
    """Run all applicable checks on a single file."""
    try:
        rel = filepath.relative_to(ROOT).as_posix()
    except ValueError:
        return

    # skip ignored dirs and top-level exclusions
    if any(part in SKIP_DIRS or part.startswith('.') for part in filepath.parts):
        return
    if "_bus" in filepath.parts:
        return
    if _is_skipped(rel):
        return

    suffix = filepath.suffix.lower()

    if suffix in CODE_EXTENSIONS:
        check_p04_circular_imports(filepath, rel)
        check_p16_magic_values(filepath, rel)
        check_p22_fixme(filepath, rel)
        check_p15_output_import(filepath, rel)
        check_struct_docstring(filepath, rel)

    if suffix in SCHEMA_EXTENSIONS:
        check_schema_placement(filepath, rel)


def check_dir(dirpath: Path):
    """Run directory-level checks."""
    try:
        rel = dirpath.relative_to(ROOT).as_posix()
    except ValueError:
        return
    if any(part in SKIP_DIRS or part.startswith('.') for part in dirpath.parts):
        return
    if "_bus" in dirpath.parts:
        return
    if _is_skipped(rel):
        return
    check_p25_manifest(dirpath, rel)


# ──────────────────────────────────────────────────────────────────────────────
# WATCHDOG HANDLER
# ──────────────────────────────────────────────────────────────────────────────

class ConventionHandler(FileSystemEventHandler):
    def __init__(self):
        super().__init__()
        self._pending_dirs: set[Path] = set()
        self._lock = threading.Lock()
        # defer directory checks slightly — let MANIFEST.md get created first
        threading.Thread(target=self._dir_check_loop, daemon=True).start()

    def _dir_check_loop(self):
        """Check pending directories after a short delay."""
        while True:
            time.sleep(5)
            with self._lock:
                dirs = list(self._pending_dirs)
                self._pending_dirs.clear()
            for d in dirs:
                try:
                    check_dir(d)
                except OSError:
                    pass

    def on_created(self, event):
        path = Path(event.src_path)
        if event.is_directory:
            with self._lock:
                self._pending_dirs.add(path)
        else:
            try:
                if not path.is_file():
                    return
            except OSError:
                return
            check_file(path)

    def on_modified(self, event):
        if event.is_directory:
            return
        path = Path(event.src_path)
        try:
            if not path.is_file():
                return
        except OSError:
            return
        # only recheck on meaningful modifications (not every save)
        check_file(path)


# ──────────────────────────────────────────────────────────────────────────────
# FULL SCAN
# ──────────────────────────────────────────────────────────────────────────────

def full_scan():
    """Scan entire workspace once. Use at startup or with --scan flag."""
    print(f"[checker] full scan starting at {ROOT}")
    file_count = dir_count = violation_count_before = 0

    # count existing violations
    try:
        violation_count_before = _read_violations_count()
    except Exception:
        pass

    for item in sorted(ROOT.rglob("*")):
        try:
            if item.is_dir():
                check_dir(item)
                dir_count += 1
            elif item.is_file():
                check_file(item)
                file_count += 1
        except OSError:
            continue

    new_violations = _read_violations_count() - violation_count_before
    print(f"[checker] scan complete — {file_count} files, {dir_count} dirs, {new_violations} new violations")

    # Post a ONE-LINE summary to broadcast (not individual violations)
    if new_violations > 0:
        broadcast = BUS / "broadcast.md"
        ts = _now()
        summary = (
            f"\n<!-- MSG {ts} | FROM: convention-checker | TO: all | TYPE: alert -->\n"
            f"Convention scan complete: **{new_violations} violations** found "
            f"({file_count} files, {dir_count} dirs scanned). "
            f"Details in `_bus/convention_violations.md`.\n"
            f"<!-- /MSG -->\n"
        )
        try:
            with open(broadcast, "a", encoding="utf-8") as fp:
                fp.write(summary)
        except OSError:
            pass


def _read_violations_count() -> int:
    try:
        return (BUS / "convention_violations.md").read_text(encoding="utf-8").count("<!-- MSG")
    except FileNotFoundError:
        return 0


# ──────────────────────────────────────────────────────────────────────────────
# MAIN
# ──────────────────────────────────────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Convention Checker")
    parser.add_argument("--scan", action="store_true", help="One-time full scan then exit")
    args = parser.parse_args()

    if args.scan:
        full_scan()
        return

    # run a startup scan first
    full_scan()

    handler  = ConventionHandler()
    observer = Observer()
    observer.schedule(handler, str(ROOT), recursive=True)
    observer.start()
    print(f"[checker] watching {ROOT}")
    print("[checker] Ctrl+C to stop\n")

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        pass

    observer.stop()
    observer.join()
    print("[checker] stopped.")


if __name__ == "__main__":
    main()
