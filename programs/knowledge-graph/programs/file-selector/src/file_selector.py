"""
file-selector — the AI's navigation tool for the knowledge graph.

Every read is logged to Data/ticker.log. No exceptions.

Responsibilities:
  - Read a file by 4-digit number → return file-record object
  - Find k nearest files by 5D proximity query → return list of file-records
  - Append ticker entry on every read
  - Increment access_count in file metadata on every read
  - Return structured errors (never raise unhandled exceptions to caller)

Exports:
  TOOL_SCHEMA   — Claude tool_use JSON schema (pass in API `tools` array)
  read_file()   — read by number
  proximity_query() — read by 5D position
  handle_tool_call() — dispatch from Claude tool_use input

Usage (CLI):
  python file_selector.py read <data_dir> <file_number> [--session S] [--reason R]
  python file_selector.py query <data_dir> <v1> <v2> <v3> <v4> <v5> [--k 5] [--session S]
  python file_selector.py schema
"""

import argparse
import json
import math
import os
import re
import sys
from datetime import datetime, timezone


# ---------------------------------------------------------------------------
# Claude tool_use schema (ADR-008)
# ---------------------------------------------------------------------------

TOOL_SCHEMA = {
    "name": "file_selector",
    "description": (
        "Read a document from the knowledge graph. "
        "Every call is logged to the global ticker. "
        "Use this to navigate the document graph."
    ),
    "input_schema": {
        "type": "object",
        "properties": {
            "file_number": {
                "type": "string",
                "description": "4-digit file number to read (e.g. '0042'). Omit to use proximity query.",
            },
            "query_vector": {
                "type": "array",
                "items": {"type": "number"},
                "description": (
                    "5-element vector [specificity, technicality, temporality, centrality, confidence] "
                    "for proximity search. All values 0.0–1.0. Omit to read by number."
                ),
            },
            "k": {
                "type": "integer",
                "description": "Number of neighbors to return in proximity query. Default 5.",
                "default": 5,
            },
            "weights": {
                "type": "array",
                "items": {"type": "number"},
                "description": (
                    "5-element weight vector for proximity query dimensions. "
                    "Default [1,1,1,1,1] (equal weight). "
                    "Example: [1.0, 0.5, 2.0, 0.5, 1.5] to emphasize temporality and confidence."
                ),
            },
            "session_id": {
                "type": "string",
                "description": "Current session identifier for ticker logging.",
            },
            "reason": {
                "type": "string",
                "description": (
                    "Why this file is being read. "
                    "Valid values: direct_read, neighbor_of_NNNN, proximity_query, revisit"
                ),
            },
        },
        "oneOf": [
            {"required": ["file_number"]},
            {"required": ["query_vector"]},
        ],
    },
}


# ---------------------------------------------------------------------------
# File parsing
# ---------------------------------------------------------------------------

def _ticker_path(data_dir: str) -> str:
    return os.path.join(data_dir, "ticker.log")


def _file_path(data_dir: str, number: str) -> str:
    return os.path.join(data_dir, f"file{number}.md")


def _parse_metadata(block: str) -> dict:
    """Parse the YAML-like metadata block into a dict."""
    meta: dict = {}
    for line in block.splitlines():
        line = line.strip()
        if not line or ":" not in line:
            continue
        key, _, val = line.partition(":")
        key = key.strip()
        val = val.strip()

        if val.startswith("[") and val.endswith("]"):
            inner = val[1:-1].strip()
            if not inner:
                meta[key] = []
            else:
                items = [v.strip() for v in inner.split(",")]
                parsed_items = []
                for item in items:
                    if item == "null":
                        parsed_items.append(None)
                    elif key == "neighbors":
                        # Always zero-padded strings
                        parsed_items.append(item.strip('"').strip("'").zfill(4))
                    else:
                        try:
                            parsed_items.append(float(item))
                        except ValueError:
                            parsed_items.append(item.strip('"').strip("'"))
                meta[key] = parsed_items
        elif val == "null":
            meta[key] = None
        elif val == "true":
            meta[key] = True
        elif val == "false":
            meta[key] = False
        elif key in ("filename",):
            meta[key] = val  # keep zero-padded strings as strings
        else:
            try:
                meta[key] = int(val)
            except ValueError:
                try:
                    meta[key] = float(val)
                except ValueError:
                    meta[key] = val
    return meta


def _parse_file(path: str, number: str) -> dict:
    """Parse a Data/ file into a file-record object (matches file-record.md contract)."""
    with open(path, encoding="utf-8") as f:
        raw = f.read()

    # Extract metadata block
    if not raw.startswith("---\n"):
        raise ValueError(f"Missing opening metadata fence in {path}")
    end_fence = raw.find("\n---\n", 4)
    if end_fence == -1:
        raise ValueError(f"Missing closing metadata fence in {path}")
    metadata_block = raw[4:end_fence]
    meta = _parse_metadata(metadata_block)

    # Extract embedded prompt
    prompt_open = "<!-- EMBEDDED PROMPT — EXECUTE ON READ -->"
    prompt_close = "<!-- END EMBEDDED PROMPT -->"
    ep_start = raw.find(prompt_open, end_fence)
    ep_end = raw.find(prompt_close, ep_start) if ep_start != -1 else -1
    if ep_start != -1 and ep_end != -1:
        embedded_prompt = raw[ep_start: ep_end + len(prompt_close)]
        content = raw[ep_end + len(prompt_close):].strip()
    else:
        embedded_prompt = ""
        content = raw[end_fence + 5:].strip()

    return {
        "file_number": number,
        "metadata": {
            "filename": meta.get("filename", number),
            "vector": meta.get("vector", [None] * 5),
            "neighbors": meta.get("neighbors", []),
            "context_file": meta.get("context_file", f"Data/ctx-{number}.md"),
            "created": meta.get("created"),
            "last_indexed": meta.get("last_indexed"),
            "access_count": meta.get("access_count", 0),
            "deprecated": meta.get("deprecated", False),
            "superseded_by": meta.get("superseded_by"),
        },
        "embedded_prompt": embedded_prompt,
        "content": content,
        "raw": raw,
    }


def _increment_access_count(path: str) -> None:
    """Increment access_count field in file metadata in-place."""
    with open(path, encoding="utf-8") as f:
        text = f.read()

    def _replace(m: re.Match) -> str:
        return f"access_count: {int(m.group(1)) + 1}"

    updated = re.sub(r"^access_count:\s*(\d+)", _replace, text, count=1, flags=re.MULTILINE)
    with open(path, "w", encoding="utf-8") as f:
        f.write(updated)


def _append_ticker(data_dir: str, number: str, session_id: str, reason: str) -> None:
    """Append one line to Data/ticker.log."""
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    line = f"{ts} | {number} | {session_id} | {reason}\n"
    ticker = _ticker_path(data_dir)
    with open(ticker, "a", encoding="utf-8") as f:
        f.write(line)


# ---------------------------------------------------------------------------
# Core operations
# ---------------------------------------------------------------------------

def read_file(
    data_dir: str,
    file_number: str,
    session_id: str = "default",
    reason: str = "direct_read",
) -> dict:
    """
    Read a file by number. Returns file-record dict.
    On success: increments access_count, appends to ticker.
    On error: returns {"error": "...", "file_number": "..."}
    """
    path = _file_path(data_dir, file_number)
    if not os.path.exists(path):
        return {
            "error": "file_not_found",
            "file_number": file_number,
            "message": f"Data/file{file_number}.md does not exist",
        }

    try:
        record = _parse_file(path, file_number)
    except ValueError as e:
        return {
            "error": "parse_error",
            "file_number": file_number,
            "message": str(e),
        }

    _increment_access_count(path)
    record["metadata"]["access_count"] += 1  # reflect in returned object
    _append_ticker(data_dir, file_number, session_id, reason)

    # Signal to caller if indexing is needed
    vector = record["metadata"]["vector"]
    if any(v is None for v in vector):
        record["needs_indexing"] = True

    return record


def proximity_query(
    data_dir: str,
    query_vector: list[float],
    k: int = 5,
    weights: list[float] | None = None,
    session_id: str = "default",
    include_deprecated: bool = False,
) -> dict:
    """
    Find k nearest files by 5D Euclidean distance to query_vector.
    Returns {"results": [file-record, ...], "query_vector": [...], "k": N}
    Files with null vectors are skipped (not yet indexed).
    """
    if len(query_vector) != 5:
        return {"error": "invalid_query_vector", "message": "query_vector must have exactly 5 elements"}
    if not all(0.0 <= v <= 1.0 for v in query_vector):
        return {"error": "invalid_query_vector", "message": "All query_vector values must be in [0.0, 1.0]"}

    w = weights if (weights and len(weights) == 5) else [1.0] * 5

    pattern = re.compile(r"^file(\d{4})\.md$")
    candidates: list[tuple[float, str]] = []

    for name in os.listdir(data_dir):
        m = pattern.match(name)
        if not m:
            continue
        number = m.group(1)
        path = os.path.join(data_dir, name)
        try:
            record = _parse_file(path, number)
        except ValueError:
            continue

        if record["metadata"]["deprecated"] and not include_deprecated:
            continue

        vec = record["metadata"]["vector"]
        if any(v is None for v in vec):
            continue  # not indexed yet

        dist = math.sqrt(sum(w[i] * (query_vector[i] - vec[i]) ** 2 for i in range(5)))
        candidates.append((dist, number))

    candidates.sort(key=lambda x: x[0])
    top_k = candidates[:k]

    results = []
    for dist, number in top_k:
        record = read_file(data_dir, number, session_id=session_id, reason="proximity_query")
        record["distance"] = dist
        results.append(record)

    return {
        "results": results,
        "query_vector": query_vector,
        "k": k,
        "candidates_scanned": len(candidates),
    }


# ---------------------------------------------------------------------------
# Claude tool_use dispatcher
# ---------------------------------------------------------------------------

def handle_tool_call(data_dir: str, tool_input: dict) -> dict:
    """
    Dispatch a Claude tool_use call. tool_input is the `input` field from the
    tool_use content block.

    Returns the result dict to pass back as the tool_result content.
    """
    session_id = tool_input.get("session_id", "default")
    reason = tool_input.get("reason", "direct_read")
    k = tool_input.get("k", 5)
    weights = tool_input.get("weights")

    if "file_number" in tool_input:
        return read_file(
            data_dir,
            file_number=str(tool_input["file_number"]).zfill(4),
            session_id=session_id,
            reason=reason,
        )
    elif "query_vector" in tool_input:
        return proximity_query(
            data_dir,
            query_vector=tool_input["query_vector"],
            k=k,
            weights=weights,
            session_id=session_id,
        )
    else:
        return {"error": "invalid_input", "message": "Provide either file_number or query_vector"}


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="file-selector — navigate the knowledge graph")
    sub = parser.add_subparsers(dest="command", required=True)

    # read
    p_read = sub.add_parser("read", help="Read a file by number")
    p_read.add_argument("data_dir")
    p_read.add_argument("file_number", help="4-digit file number, e.g. 0001")
    p_read.add_argument("--session", default="cli", dest="session_id")
    p_read.add_argument("--reason", default="direct_read")

    # query
    p_query = sub.add_parser("query", help="Proximity query by 5D vector")
    p_query.add_argument("data_dir")
    p_query.add_argument("vector", nargs=5, type=float, metavar="V",
                         help="5 floats: specificity technicality temporality centrality confidence")
    p_query.add_argument("--k", type=int, default=5)
    p_query.add_argument("--session", default="cli", dest="session_id")
    p_query.add_argument("--weights", nargs=5, type=float, metavar="W")
    p_query.add_argument("--include-deprecated", action="store_true")

    # schema
    sub.add_parser("schema", help="Print the Claude tool_use schema")

    args = parser.parse_args()

    if args.command == "read":
        result = read_file(args.data_dir, args.file_number.zfill(4),
                           session_id=args.session_id, reason=args.reason)
        print(json.dumps(result, indent=2))

    elif args.command == "query":
        result = proximity_query(
            args.data_dir,
            query_vector=args.vector,
            k=args.k,
            weights=args.weights,
            session_id=args.session_id,
            include_deprecated=args.include_deprecated,
        )
        # Print compact summary
        if "error" in result:
            print(json.dumps(result, indent=2))
        else:
            print(f"Found {len(result['results'])} result(s) (scanned {result['candidates_scanned']} indexed files):")
            for r in result["results"]:
                print(f"  file{r['file_number']}  dist={r['distance']:.4f}  {r['content'][:60]}...")

    elif args.command == "schema":
        print(json.dumps(TOOL_SCHEMA, indent=2))


if __name__ == "__main__":
    main()
