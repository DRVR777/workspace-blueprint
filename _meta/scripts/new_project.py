"""
new_project.py — Clone the template and create a new project.

Usage:
    python new_project.py <project-name>
    python new_project.py <project-name> --prd "prd text here"
    python new_project.py <project-name> --prd @path/to/prd.md

What it does:
1. Checks if programs/<project-name> already exists and is active (stops if so)
2. Copies programs/_template/ to programs/<project-name>/
3. Replaces {{PROJECT_NAME}} and {{CREATED}} throughout all files
4. Writes PRD to programs/<project-name>/_planning/prd-source.md (if provided)
5. Writes PRD to _intake/queue/<project-name>-prd.md (for agent to process)
6. Prints next steps

The template is never modified. It always stays as the clean clone source.
"""

import os
import sys
import shutil
import re
from datetime import datetime, timezone

# Workspace root = 3 levels up from this script (_meta/scripts/new_project.py)
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIR))
TEMPLATE_PATH = os.path.join(WORKSPACE_ROOT, "programs", "_template")
PROGRAMS_PATH = os.path.join(WORKSPACE_ROOT, "programs")
INTAKE_QUEUE = os.path.join(WORKSPACE_ROOT, "_intake", "queue")


def slugify(name: str) -> str:
    """Convert a project name to lowercase-hyphenated slug."""
    name = name.lower().strip()
    name = re.sub(r"[^a-z0-9\s-]", "", name)
    name = re.sub(r"[\s_]+", "-", name)
    name = re.sub(r"-+", "-", name)
    return name.strip("-")


def is_active_project(project_path: str) -> bool:
    """Return True if the project exists and has active/specced/complete status."""
    manifest = os.path.join(project_path, "MANIFEST.md")
    if not os.path.exists(manifest):
        return False
    with open(manifest, "r", encoding="utf-8") as f:
        content = f.read()
    # Scaffold is OK to overwrite; active/specced/complete are not
    for status in ("active", "specced", "complete"):
        if f"| `status` | {status}" in content:
            return True
    return False


def replace_placeholders(path: str, project_name: str, timestamp: str) -> None:
    """Walk the cloned project and replace all {{PLACEHOLDER}} values."""
    for root, dirs, files in os.walk(path):
        # Don't descend into .git
        dirs[:] = [d for d in dirs if d != ".git"]
        for filename in files:
            if not any(filename.endswith(ext) for ext in (".md", ".txt", ".json", ".py")):
                continue
            filepath = os.path.join(root, filename)
            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
                content = content.replace("{{PROJECT_NAME}}", project_name)
                content = content.replace("{{CREATED}}", timestamp)
                with open(filepath, "w", encoding="utf-8") as f:
                    f.write(content)
            except (UnicodeDecodeError, PermissionError):
                pass  # Skip binary or locked files


def create_project(project_name: str, prd_text: str | None = None) -> bool:
    slug = slugify(project_name)
    target = os.path.join(PROGRAMS_PATH, slug)
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    # Guard: template must exist
    if not os.path.exists(TEMPLATE_PATH):
        print(f"ERROR: Template not found at {TEMPLATE_PATH}")
        print("The template must exist at programs/_template/ before creating projects.")
        return False

    # Guard: don't overwrite an active project
    if is_active_project(target):
        print(f"ERROR: Project '{slug}' already exists and is active.")
        print("Choose a different name, or manually set status to 'scaffold' to re-scaffold.")
        return False

    # Remove stale scaffold if present
    if os.path.exists(target):
        print(f"Found existing scaffold at programs/{slug}/ — removing and re-cloning.")
        shutil.rmtree(target)

    # Clone template
    shutil.copytree(TEMPLATE_PATH, target)
    replace_placeholders(target, slug, timestamp)

    print(f"Created programs/{slug}/")

    # Write PRD to project planning folder
    if prd_text:
        prd_dir = os.path.join(target, "_planning")
        os.makedirs(prd_dir, exist_ok=True)
        prd_path = os.path.join(prd_dir, "prd-source.md")
        with open(prd_path, "w", encoding="utf-8") as f:
            f.write(f"# PRD Source — {slug}\n\nReceived: {timestamp}\n\n---\n\n{prd_text}")
        print(f"PRD written to programs/{slug}/_planning/prd-source.md")

        # Also drop in intake queue for agent processing
        os.makedirs(INTAKE_QUEUE, exist_ok=True)
        queue_path = os.path.join(INTAKE_QUEUE, f"{slug}-prd.md")
        with open(queue_path, "w", encoding="utf-8") as f:
            f.write(prd_text)
        print(f"PRD queued at _intake/queue/{slug}-prd.md")

    print()
    print("Next steps:")
    print(f"  1. Open programs/{slug}/_planning/adr/ — validate every 'assumption' ADR")
    print(f"  2. Open programs/{slug}/shared/contracts/ — define shape of every stub contract")
    print(f"  3. Run _meta/prd-intake.md to populate programs and contracts from the PRD")
    print(f"  4. Run _meta/spec-review.md on each program before building")
    return True


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: python new_project.py <project-name> [--prd 'text' | --prd @file.md]")
        sys.exit(1)

    project_name = sys.argv[1]
    prd_text = None

    if "--prd" in sys.argv:
        idx = sys.argv.index("--prd") + 1
        if idx >= len(sys.argv):
            print("ERROR: --prd flag requires a value")
            sys.exit(1)
        prd_arg = sys.argv[idx]
        if prd_arg.startswith("@"):
            filepath = prd_arg[1:]
            if not os.path.exists(filepath):
                print(f"ERROR: PRD file not found: {filepath}")
                sys.exit(1)
            with open(filepath, "r", encoding="utf-8") as f:
                prd_text = f.read()
        else:
            prd_text = prd_arg

    success = create_project(project_name, prd_text)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
