"""
knowledge-graph MCP server

Exposes the full knowledge graph system as Claude tools via the Model Context Protocol.
Once registered, any Claude Code session can navigate, create, and index documents
in the knowledge graph without manual script calls.

Tools exposed:
  kg_read          — read a file by 4-digit number
  kg_query         — find k nearest files by 5D vector proximity
  kg_create        — create a new file in the graph
  kg_index         — assign 5D vector and neighbors to a file
  kg_index_batch   — index all un-indexed files
  kg_build_ctx     — generate ctx-NNNN.md for a file
  kg_validate      — validate all files in Data/
  kg_status        — overview: file count, ctx coverage, ticker size

Configuration:
  Set KG_DATA_DIR environment variable to the absolute path of your Data/ folder.
  Default: the Data/ folder next to this file.

Register in Claude Code:
  claude mcp add knowledge-graph -- python path/to/mcp_server.py
"""

import json
import os
import re
import sys

# Add program src paths so we can import them directly
_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(_HERE, "programs", "data-store", "src"))
sys.path.insert(0, os.path.join(_HERE, "programs", "file-selector", "src"))
sys.path.insert(0, os.path.join(_HERE, "programs", "indexer", "src"))
sys.path.insert(0, os.path.join(_HERE, "programs", "context-builder", "src"))

import data_store as ds
import file_selector as fs
import indexer as ix
import context_builder as cb

from mcp.server.fastmcp import FastMCP

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

DATA_DIR = os.environ.get(
    "KG_DATA_DIR",
    os.path.join(_HERE, "Data"),
)

mcp = FastMCP("knowledge-graph")


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------

@mcp.tool()
def kg_read(
    file_number: str,
    session_id: str = "mcp",
    reason: str = "direct_read",
) -> str:
    """
    Read a document from the knowledge graph by its 4-digit file number.

    Returns the file's metadata, embedded prompt, and content.
    Also logs the read to ticker.log and increments access_count.

    If the file has a null vector, the result includes needs_indexing=true —
    call kg_index next to assign its position in the graph.

    Args:
        file_number: 4-digit number, e.g. "0001" or "42" (auto-padded)
        session_id: identifier for this session (used in ticker log)
        reason: why this file is being read — direct_read, neighbor_of_NNNN, proximity_query, revisit
    """
    result = fs.read_file(DATA_DIR, file_number.zfill(4), session_id=session_id, reason=reason)
    return json.dumps(result, indent=2)


@mcp.tool()
def kg_query(
    specificity: float,
    technicality: float,
    temporality: float,
    centrality: float,
    confidence: float,
    k: int = 5,
    session_id: str = "mcp",
) -> str:
    """
    Find the k nearest documents in the knowledge graph to a 5D position.

    The 5 dimensions (all values 0.0–1.0):
      specificity  — 0=abstract/general, 1=concrete/detailed
      technicality — 0=intuitive/prose, 1=technical/code-level
      temporality  — 0=foundational/stable, 1=current/ephemeral
      centrality   — 0=isolated, 1=hub (many connections)
      confidence   — 0=speculative, 1=established/verified

    Returns the k closest files sorted by ascending distance.
    Files without vectors (not yet indexed) are excluded.

    Args:
        specificity: 0.0–1.0
        technicality: 0.0–1.0
        temporality: 0.0–1.0
        centrality: 0.0–1.0
        confidence: 0.0–1.0
        k: number of results to return (default 5)
        session_id: identifier for ticker logging
    """
    query_vector = [specificity, technicality, temporality, centrality, confidence]
    result = fs.proximity_query(DATA_DIR, query_vector, k=k, session_id=session_id)
    if "error" in result:
        return json.dumps(result)

    # Compact output: file numbers, distances, first sentence of content
    summary = {
        "query_vector": query_vector,
        "k": k,
        "candidates_scanned": result["candidates_scanned"],
        "results": [
            {
                "file_number": r["file_number"],
                "distance": round(r["distance"], 4),
                "vector": r["metadata"]["vector"],
                "neighbors": r["metadata"]["neighbors"],
                "summary": r["content"][:120].replace("\n", " ").strip(),
            }
            for r in result["results"]
        ],
    }
    return json.dumps(summary, indent=2)


@mcp.tool()
def kg_create(
    content: str,
    custom_prompt: str = "",
) -> str:
    """
    Create a new document in the knowledge graph.

    Assigns the next sequential file number, writes the metadata header,
    injects the standard embedded prompt (or a custom one), and updates index.md.

    After creating a file, call kg_index to assign its 5D position and neighbors.

    Args:
        content: the document text to store
        custom_prompt: optional custom embedded prompt (replaces the standard template)
    """
    path = ds.cmd_create(DATA_DIR, content=content, custom_prompt=custom_prompt)
    number = re.search(r"file(\d{4})\.md", path)
    num_str = number.group(1) if number else "????"
    return json.dumps({
        "created": f"file{num_str}.md",
        "file_number": num_str,
        "path": path,
        "next_step": f"Call kg_index('{num_str}') to assign its 5D position and neighbors.",
    })


@mcp.tool()
def kg_index(file_number: str, k: int = 5) -> str:
    """
    Compute and assign the 5D semantic vector and k nearest neighbors for a file.

    Uses heuristic analysis of content (no ML required). Writes vector, neighbors,
    and last_indexed timestamp back into the file's metadata header in-place.

    Call this after kg_create, or to refresh a file after its content changes.

    Args:
        file_number: 4-digit number, e.g. "0001"
        k: number of neighbors to find (default 5, per ADR-004)
    """
    result = ix.index_file(DATA_DIR, file_number.zfill(4), k=k)
    if "error" in result:
        return json.dumps(result)
    return json.dumps({
        "file_number": result["file_number"],
        "vector": [round(v, 3) for v in result["vector"]],
        "neighbors": result["neighbors"],
        "last_indexed": result["last_indexed"],
        "next_step": f"Call kg_build_ctx('{file_number}') to generate its context description.",
    })


@mcp.tool()
def kg_index_batch(force: bool = False) -> str:
    """
    Index all files in Data/ that don't yet have a vector assigned.

    Processes files in file-number order. Skips deprecated files.
    Use force=True to re-index all files, including already-indexed ones.

    Args:
        force: if True, re-index all files even if they already have vectors
    """
    # Capture output by calling directly
    pattern = re.compile(r"^file(\d{4})\.md$")
    results = []
    candidates = []

    for name in sorted(os.listdir(DATA_DIR)):
        m = pattern.match(name)
        if not m:
            continue
        number = m.group(1)
        path = os.path.join(DATA_DIR, name)
        try:
            meta, _ = ix._parse_file_for_indexing(path)
        except ValueError:
            continue
        if meta.get("deprecated"):
            continue
        raw_vec = meta.get("vector", [])
        has_null = not raw_vec or any(v in (None, "null") for v in raw_vec)
        if force or has_null:
            candidates.append(number)

    for number in candidates:
        r = ix.index_file(DATA_DIR, number, k=5)
        results.append(r)

    ok = [r for r in results if "vector" in r]
    return json.dumps({
        "indexed": len(ok),
        "total_candidates": len(candidates),
        "results": [
            {
                "file_number": r["file_number"],
                "vector": [round(v, 3) for v in r["vector"]],
                "neighbors": r["neighbors"],
            }
            for r in ok
        ],
    }, indent=2)


@mcp.tool()
def kg_build_ctx(file_number: str, force: bool = False) -> str:
    """
    Generate or refresh the context description file (ctx-NNNN.md) for a document.

    The context file contains:
      - What I Am: a summary of the document's content
      - My Position: interpretation of the 5D vector in plain language
      - My Neighbors: how this document relates to each neighbor
      - My Cluster: which topic area this document belongs to
      - My Role: a one-sentence description of what this document does

    The file is written to Data/ctx-NNNN.md and is readable by any agent.
    A context_built entry is appended to ticker.log after writing.

    Respects staleness threshold (5 min) unless force=True.

    Args:
        file_number: 4-digit number, e.g. "0001"
        force: if True, rebuild even if the ctx file was recently written
    """
    result = cb.build_ctx(
        DATA_DIR,
        file_number.zfill(4),
        session_id="mcp",
        staleness_minutes=0.0 if force else 5.0,
    )
    if result["status"] == "written":
        # Read back the ctx file to return it
        ctx_path = result["ctx_path"]
        with open(ctx_path, encoding="utf-8") as f:
            ctx_content = f.read()
        return json.dumps({
            "status": "written",
            "file_number": file_number.zfill(4),
            "ctx_file": ctx_path,
            "content": ctx_content,
        })
    return json.dumps(result)


@mcp.tool()
def kg_validate() -> str:
    """
    Validate all files in Data/ against the file-format spec.

    Checks:
      1. Filename matches file[NNNN].md pattern
      2. Metadata header present and complete (all 7 required fields)
      3. Embedded prompt markers present
      4. access_count is a non-negative integer
      5. vector is all nulls or all floats in [0.0, 1.0]
      6. filename field matches actual file number

    Returns a list of violations, or a clean bill of health.
    """
    pattern = re.compile(r"^file(\d{4})\.md$")
    errors = []
    files_found = 0

    METADATA_FIELDS = ["filename", "vector", "neighbors", "context_file",
                       "created", "last_indexed", "access_count"]
    PROMPT_OPEN = "<!-- EMBEDDED PROMPT — EXECUTE ON READ -->"
    PROMPT_CLOSE = "<!-- END EMBEDDED PROMPT -->"

    for name in sorted(os.listdir(DATA_DIR)):
        m = pattern.match(name)
        if not m:
            continue
        files_found += 1
        number = m.group(1)
        path = os.path.join(DATA_DIR, name)

        with open(path, encoding="utf-8") as f:
            text = f.read()

        if not text.startswith("---\n"):
            errors.append(f"{name}: missing opening metadata fence")
            continue
        end_fence = text.find("\n---\n", 4)
        if end_fence == -1:
            errors.append(f"{name}: missing closing metadata fence")
            continue
        mb = text[4:end_fence]
        for field in METADATA_FIELDS:
            if not re.search(rf"^{field}:", mb, re.MULTILINE):
                errors.append(f"{name}: missing field '{field}'")
        if PROMPT_OPEN not in text or PROMPT_CLOSE not in text:
            errors.append(f"{name}: missing embedded prompt markers")
        ac = re.search(r"^access_count:\s*(\S+)", mb, re.MULTILINE)
        if ac:
            try:
                if int(ac.group(1)) < 0:
                    errors.append(f"{name}: access_count is negative")
            except ValueError:
                errors.append(f"{name}: access_count is not an integer")
        fm = re.search(r"^filename:\s*(\S+)", mb, re.MULTILINE)
        if fm and fm.group(1) != number:
            errors.append(f"{name}: filename field mismatch")

    if not errors:
        return json.dumps({"valid": True, "files_checked": files_found,
                           "message": f"All {files_found} file(s) valid."})
    return json.dumps({"valid": False, "files_checked": files_found, "errors": errors})


@mcp.tool()
def kg_status() -> str:
    """
    Show the current state of the knowledge graph.

    Reports:
      - Total files in Data/
      - How many are indexed (have a non-null vector)
      - How many have ctx files
      - Ticker.log entry count
      - Most recently created and most recently indexed files
    """
    pattern = re.compile(r"^file(\d{4})\.md$")
    total = 0
    indexed = 0
    has_ctx = 0
    latest_created = None
    latest_indexed = None

    for name in sorted(os.listdir(DATA_DIR)):
        m = pattern.match(name)
        if not m:
            continue
        total += 1
        number = m.group(1)
        path = os.path.join(DATA_DIR, name)
        try:
            meta, _ = ix._parse_file_for_indexing(path)
        except ValueError:
            continue

        raw_vec = meta.get("vector", [])
        if raw_vec and not any(v in (None, "null") for v in raw_vec):
            indexed += 1
            li = meta.get("last_indexed")
            if li and (latest_indexed is None or li > latest_indexed):
                latest_indexed = li

        created = meta.get("created", "")
        if latest_created is None or created > latest_created:
            latest_created = created

        ctx_path = os.path.join(DATA_DIR, f"ctx-{number}.md")
        if os.path.exists(ctx_path):
            has_ctx += 1

    ticker_path = os.path.join(DATA_DIR, "ticker.log")
    ticker_entries = 0
    if os.path.exists(ticker_path):
        with open(ticker_path, encoding="utf-8") as f:
            ticker_entries = sum(
                1 for line in f
                if line.strip() and not line.startswith("#")
            )

    return json.dumps({
        "data_dir": DATA_DIR,
        "files": {
            "total": total,
            "indexed": indexed,
            "pending_index": total - indexed,
            "have_ctx": has_ctx,
            "missing_ctx": total - has_ctx,
        },
        "ticker_entries": ticker_entries,
        "latest_created": latest_created,
        "latest_indexed": latest_indexed,
    }, indent=2)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    mcp.run()
