"""
data-store — owns and manages the Data/ folder.

Responsibilities:
  - Create files with correct naming (file0001.md, file0002.md, ...)
  - Write metadata header + standard embedded prompt + content
  - Maintain Data/index.md (one row per file)
  - Maintain counter (no gaps, no duplicates)
  - Validate existing files against the file-format-spec

Usage:
  python data_store.py init <data_dir>
      Bootstrap Data/ folder with index.md and ticker.log.

  python data_store.py create <data_dir> [--content "..."] [--prompt "..."]
      Create the next file in Data/. Prints the new file path.

  python data_store.py validate <data_dir>
      Check all files for spec compliance. Prints any violations.

  python data_store.py deprecate <data_dir> <file_number> <superseded_by>
      Mark a file deprecated and update index.md.
"""

import argparse
import os
import re
import sys
from datetime import date, timezone, datetime


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

METADATA_FIELDS = [
    "filename", "vector", "neighbors", "context_file",
    "created", "last_indexed", "access_count",
]

EMBEDDED_PROMPT_OPEN  = "<!-- EMBEDDED PROMPT — EXECUTE ON READ -->"
EMBEDDED_PROMPT_CLOSE = "<!-- END EMBEDDED PROMPT -->"

STANDARD_EMBEDDED_PROMPT_TEMPLATE = """\
<!-- EMBEDDED PROMPT — EXECUTE ON READ -->
You are reading file {NNNN} in a self-navigating knowledge graph.

When you read this file, do the following:

1. CHECK VECTOR: If my vector field shows nulls, call indexer to compute my position.
   Do this before proceeding.

2. READ NEIGHBORS: My neighbors are listed above. For each neighbor number:
   Call file-selector with that number. Read the returned content.
   If I have no neighbors yet, skip to step 3.

3. WRITE/UPDATE CONTEXT FILE: Write or overwrite Data/ctx-{NNNN}.md with:

   ## What I Am
   [1 paragraph: what this document is about]

   ## My Position
   Vector: [values] — interpreted as:
   - Specificity: [low/medium/high]
   - Technicality: [low/medium/high]
   - Temporality: [stable/mixed/current]
   - Centrality: [peripheral/connected/hub]
   - Confidence: [speculative/probable/established]

   ## My Neighbors and How I Relate to Them
   [For each neighbor: file number, one-sentence relationship]

   ## My Cluster
   [One label for the topic area I belong to]

   ## My Role
   "This document is a [noun] that [verb phrase]."

4. LOG: After writing ctx-{NNNN}.md, append to Data/ticker.log:
   [timestamp] | {NNNN} | [session] | context_built
<!-- END EMBEDDED PROMPT -->"""

INDEX_HEADER = """\
# Data/ Index

| File | Created | Vector Status | One-Line Summary |
|------|---------|--------------|-----------------|
"""


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _data_dir(path: str) -> str:
    return os.path.abspath(path)


def _index_path(data_dir: str) -> str:
    return os.path.join(data_dir, "index.md")


def _ticker_path(data_dir: str) -> str:
    return os.path.join(data_dir, "ticker.log")


def _file_path(data_dir: str, number: str) -> str:
    return os.path.join(data_dir, f"file{number}.md")


def _next_number(data_dir: str) -> str:
    """Return the next available zero-padded 4-digit file number."""
    existing = []
    pattern = re.compile(r"^file(\d{4})\.md$")
    for name in os.listdir(data_dir):
        m = pattern.match(name)
        if m:
            existing.append(int(m.group(1)))
    if not existing:
        return "0001"
    return str(max(existing) + 1).zfill(4)


def _first_sentence(text: str) -> str:
    """Extract and truncate to first sentence, max 100 chars."""
    text = text.strip()
    for sep in (".", "!", "?", "\n"):
        idx = text.find(sep)
        if idx != -1:
            text = text[: idx + 1].strip()
            break
    return text[:100] if len(text) > 100 else text


def _read_index_rows(data_dir: str) -> list[str]:
    idx = _index_path(data_dir)
    if not os.path.exists(idx):
        return []
    with open(idx, encoding="utf-8") as f:
        lines = f.readlines()
    # Return only data rows (skip header lines)
    rows = []
    in_table = False
    for line in lines:
        if line.startswith("| File |"):
            in_table = True
            continue
        if in_table and line.startswith("|---"):
            continue
        if in_table and line.startswith("|"):
            rows.append(line)
    return rows


def _write_index(data_dir: str, rows: list[str]) -> None:
    idx = _index_path(data_dir)
    with open(idx, "w", encoding="utf-8") as f:
        f.write(INDEX_HEADER)
        for row in rows:
            f.write(row if row.endswith("\n") else row + "\n")


def _append_index_row(data_dir: str, number: str, summary: str) -> None:
    rows = _read_index_rows(data_dir)
    today = date.today().isoformat()
    new_row = f"| file{number} | {today} | pending | {summary} |\n"
    rows.append(new_row)
    _write_index(data_dir, rows)


def _update_index_row(
    data_dir: str,
    number: str,
    vector_status: str | None = None,
    append_note: str | None = None,
) -> None:
    rows = _read_index_rows(data_dir)
    updated = []
    for row in rows:
        if f"| file{number} |" in row:
            parts = [p.strip() for p in row.strip().strip("|").split("|")]
            if vector_status:
                parts[2] = vector_status
            if append_note:
                parts[3] = parts[3].rstrip() + " " + append_note
            row = "| " + " | ".join(parts) + " |\n"
        updated.append(row)
    _write_index(data_dir, updated)


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

def cmd_init(data_dir: str) -> None:
    os.makedirs(data_dir, exist_ok=True)
    idx = _index_path(data_dir)
    ticker = _ticker_path(data_dir)

    if not os.path.exists(idx):
        with open(idx, "w", encoding="utf-8") as f:
            f.write(INDEX_HEADER)
        print(f"Created {idx}")
    else:
        print(f"index.md already exists — skipped")

    if not os.path.exists(ticker):
        with open(ticker, "w", encoding="utf-8") as f:
            f.write("# ticker.log — append-only read/write log\n")
        print(f"Created {ticker}")
    else:
        print(f"ticker.log already exists — skipped")

    print(f"Data/ bootstrapped at: {data_dir}")


def cmd_create(data_dir: str, content: str = "", custom_prompt: str = "") -> str:
    if not os.path.exists(data_dir):
        print(f"Error: data_dir does not exist: {data_dir}", file=sys.stderr)
        sys.exit(1)

    number = _next_number(data_dir)
    today = date.today().isoformat()

    prompt_block = (
        custom_prompt
        if custom_prompt
        else STANDARD_EMBEDDED_PROMPT_TEMPLATE.replace("{NNNN}", number)
    )

    file_body = f"""\
---
filename: {number}
vector: [null, null, null, null, null]
neighbors: []
context_file: Data/ctx-{number}.md
created: {today}
last_indexed: null
access_count: 0
---

{prompt_block}

{content}
""".lstrip()

    path = _file_path(data_dir, number)
    with open(path, "w", encoding="utf-8") as f:
        f.write(file_body)

    summary = _first_sentence(content) if content else "(no content)"
    _append_index_row(data_dir, number, summary)

    print(path)
    return path


def cmd_deprecate(data_dir: str, number: str, superseded_by: str) -> None:
    path = _file_path(data_dir, number)
    if not os.path.exists(path):
        print(f"Error: file{number}.md not found in {data_dir}", file=sys.stderr)
        sys.exit(1)

    with open(path, "r", encoding="utf-8") as f:
        text = f.read()

    # Insert deprecated + superseded_by into metadata
    if "deprecated:" in text:
        text = re.sub(r"deprecated:\s*\S+", "deprecated: true", text)
    else:
        text = text.replace("access_count:", f"deprecated: true\nsuperseded_by: {superseded_by}\naccess_count:", 1)

    if "superseded_by:" not in text:
        text = text.replace("deprecated: true", f"deprecated: true\nsuperseded_by: {superseded_by}", 1)
    else:
        text = re.sub(r"superseded_by:\s*\S+", f"superseded_by: {superseded_by}", text)

    # Replace embedded prompt with deprecation notice
    dep_prompt = f"""{EMBEDDED_PROMPT_OPEN}
This file is deprecated. Read file{superseded_by} instead.
Call file-selector("{superseded_by}") now.
{EMBEDDED_PROMPT_CLOSE}"""

    text = re.sub(
        re.escape(EMBEDDED_PROMPT_OPEN) + r".*?" + re.escape(EMBEDDED_PROMPT_CLOSE),
        dep_prompt,
        text,
        flags=re.DOTALL,
    )

    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

    _update_index_row(data_dir, number, vector_status="deprecated",
                      append_note=f"[DEPRECATED → {superseded_by}]")
    print(f"file{number}.md deprecated. Superseded by file{superseded_by}.md.")


def cmd_validate(data_dir: str) -> None:
    pattern = re.compile(r"^file(\d{4})\.md$")
    errors: list[str] = []
    files_found = 0

    for name in sorted(os.listdir(data_dir)):
        if not pattern.match(name):
            continue
        files_found += 1
        path = os.path.join(data_dir, name)
        number = pattern.match(name).group(1)

        with open(path, encoding="utf-8") as f:
            text = f.read()

        # Rule 1: metadata header present
        if not text.startswith("---\n"):
            errors.append(f"{name}: missing opening '---' metadata fence")
            continue

        end_fence = text.find("\n---\n", 4)
        if end_fence == -1:
            errors.append(f"{name}: missing closing '---' metadata fence")
            continue

        metadata_block = text[4:end_fence]

        # Rule 2: all required fields present
        for field in METADATA_FIELDS:
            if not re.search(rf"^{field}:", metadata_block, re.MULTILINE):
                errors.append(f"{name}: missing metadata field '{field}'")

        # Rule 3: embedded prompt present
        if EMBEDDED_PROMPT_OPEN not in text or EMBEDDED_PROMPT_CLOSE not in text:
            errors.append(f"{name}: missing embedded prompt markers")

        # Rule 4: access_count is non-negative int
        m = re.search(r"^access_count:\s*(\S+)", metadata_block, re.MULTILINE)
        if m:
            try:
                val = int(m.group(1))
                if val < 0:
                    errors.append(f"{name}: access_count is negative ({val})")
            except ValueError:
                errors.append(f"{name}: access_count is not an integer ('{m.group(1)}')")

        # Rule 5: vector is all nulls or all floats in [0,1]
        vm = re.search(r"^vector:\s*\[(.+)\]", metadata_block, re.MULTILINE)
        if vm:
            raw = [v.strip() for v in vm.group(1).split(",")]
            if not all(v == "null" for v in raw):
                try:
                    floats = [float(v) for v in raw]
                    if len(floats) != 5:
                        errors.append(f"{name}: vector must have exactly 5 elements")
                    elif not all(0.0 <= v <= 1.0 for v in floats):
                        errors.append(f"{name}: vector values must be in [0.0, 1.0]")
                except ValueError:
                    errors.append(f"{name}: vector contains non-float, non-null values")

        # Rule 6: filename field matches file number
        fm = re.search(r"^filename:\s*(\S+)", metadata_block, re.MULTILINE)
        if fm and fm.group(1) != number:
            errors.append(f"{name}: filename field '{fm.group(1)}' doesn't match file number '{number}'")

    if files_found == 0:
        print("No files found in Data/.")
    elif not errors:
        print(f"All {files_found} file(s) valid.")
    else:
        for e in errors:
            print(f"ERROR: {e}")
        sys.exit(1)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="data-store — manage the knowledge graph Data/ folder"
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # init
    p_init = sub.add_parser("init", help="Bootstrap Data/ folder")
    p_init.add_argument("data_dir", help="Path to Data/ folder")

    # create
    p_create = sub.add_parser("create", help="Create the next file")
    p_create.add_argument("data_dir", help="Path to Data/ folder")
    p_create.add_argument("--content", default="", help="File content text")
    p_create.add_argument("--prompt", default="", help="Custom embedded prompt (replaces standard)")

    # deprecate
    p_dep = sub.add_parser("deprecate", help="Deprecate a file")
    p_dep.add_argument("data_dir", help="Path to Data/ folder")
    p_dep.add_argument("file_number", help="Zero-padded 4-digit number, e.g. 0003")
    p_dep.add_argument("superseded_by", help="Number of the replacement file")

    # validate
    p_val = sub.add_parser("validate", help="Validate all files in Data/")
    p_val.add_argument("data_dir", help="Path to Data/ folder")

    args = parser.parse_args()

    if args.command == "init":
        cmd_init(args.data_dir)
    elif args.command == "create":
        cmd_create(args.data_dir, content=args.content, custom_prompt=args.prompt)
    elif args.command == "deprecate":
        cmd_deprecate(args.data_dir, args.file_number, args.superseded_by)
    elif args.command == "validate":
        cmd_validate(args.data_dir)


if __name__ == "__main__":
    main()
