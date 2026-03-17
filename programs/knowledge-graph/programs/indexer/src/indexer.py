"""
indexer — assigns 5D semantic vectors and k-nearest neighbors to every file.

Heuristic method (ADR-003). No ML required. Upgrade path is clean (only this file changes).

The 5 dimensions (ADR-002, 5d-vector-spec.md):
  0  Specificity  — how concrete/detailed is this document?
  1  Technicality — how technical/formal vs intuitive/prose?
  2  Temporality  — how time-sensitive? (0=foundational, 1=ephemeral)
  3  Centrality   — how connected to others? (improved by ticker.log over time)
  4  Confidence   — how established/certain is the content?

All values are floats in [0.0, 1.0].

Usage:
  python indexer.py index <data_dir> <file_number>
      Index one file. Writes vector + neighbors + last_indexed back to file.

  python indexer.py batch <data_dir>
      Index all files that have a null vector. Processes in file-number order.

  python indexer.py reindex <data_dir> <file_number>
      Force-reindex a file even if it already has a vector.

  python indexer.py neighbors <data_dir> <file_number>
      Print the current neighbors of a file without re-indexing.
"""

import argparse
import math
import os
import re
import sys
from datetime import datetime, timezone


# ---------------------------------------------------------------------------
# Heuristic signal sets
# ---------------------------------------------------------------------------

# Temporality signals — presence pushes score toward current/ephemeral
TEMPORAL_HIGH = {
    "currently", "today", "this week", "this sprint", "right now", "just",
    "recent", "latest", "update", "status", "wip", "in progress", "draft",
    "meeting", "notes", "standup", "todo",
}
TEMPORAL_LOW = {
    "always", "never", "principle", "foundation", "fundamentally", "invariant",
    "core", "canonical", "spec", "architecture", "design", "philosophy",
}

# Confidence signals
CONFIDENCE_HIGH = {
    "confirmed", "verified", "tested", "proven", "established", "accepted",
    "implemented", "production", "shipped", "done", "complete", "working",
    "validated", "approved",
}
CONFIDENCE_LOW = {
    "hypothesis", "hypothetical", "might", "maybe", "perhaps", "assume",
    "assumption", "speculative", "explore", "experiment", "draft", "brainstorm",
    "uncertain", "unclear", "tbd", "todo", "open question",
}
CONFIDENCE_MED = {
    "probably", "likely", "proposed", "candidate", "consider", "option",
    "could", "should", "planned",
}

# Technicality signals
TECHNICAL_HIGH = {
    "```", "def ", "class ", "function", "import ", "return ", "const ",
    "var ", "let ", "type ", "interface ", "struct ", "schema", "json",
    "yaml", "sql", "api", "endpoint", "protocol", "algorithm", "implementation",
    "binary", "hex", "regex", "lambda", "async", "await", "socket",
}
TECHNICAL_LOW = {
    "vision", "goal", "mission", "values", "culture", "user story",
    "persona", "journey", "feeling", "experience", "delight", "community",
}

# ---------------------------------------------------------------------------
# File parser (minimal — only needs content and metadata fields)
# ---------------------------------------------------------------------------

def _file_path(data_dir: str, number: str) -> str:
    return os.path.join(data_dir, f"file{number}.md")


def _parse_file_for_indexing(path: str) -> tuple[dict, str]:
    """Returns (metadata_dict, full_content_text)."""
    with open(path, encoding="utf-8") as f:
        raw = f.read()

    if not raw.startswith("---\n"):
        raise ValueError(f"Missing opening metadata fence: {path}")
    end_fence = raw.find("\n---\n", 4)
    if end_fence == -1:
        raise ValueError(f"Missing closing metadata fence: {path}")

    meta_block = raw[4:end_fence]
    meta: dict = {}
    for line in meta_block.splitlines():
        if ":" not in line:
            continue
        key, _, val = line.partition(":")
        key, val = key.strip(), val.strip()
        if key == "filename":
            meta[key] = val
        elif val.startswith("[") and val.endswith("]"):
            inner = val[1:-1].strip()
            meta[key] = [] if not inner else [v.strip() for v in inner.split(",")]
        elif val == "null":
            meta[key] = None
        elif val in ("true", "false"):
            meta[key] = val == "true"
        else:
            try:
                meta[key] = int(val)
            except ValueError:
                meta[key] = val

    # Content = everything after the closing fence
    content = raw[end_fence + 5:]
    # Strip embedded prompt markers for cleaner text analysis
    content = re.sub(
        r"<!-- EMBEDDED PROMPT.*?<!-- END EMBEDDED PROMPT -->",
        "",
        content,
        flags=re.DOTALL,
    ).strip()

    return meta, content


def _write_vector_back(
    path: str,
    vector: list[float],
    neighbors: list[str],
    last_indexed: str,
) -> None:
    """Update vector, neighbors, and last_indexed in a file's metadata in-place."""
    with open(path, encoding="utf-8") as f:
        text = f.read()

    vec_str = "[" + ", ".join(f"{v:.3f}" for v in vector) + "]"
    nb_str = "[" + ", ".join(neighbors) + "]"

    text = re.sub(r"^vector:\s*\[.*?\]", f"vector: {vec_str}", text, count=1, flags=re.MULTILINE)
    text = re.sub(r"^neighbors:\s*\[.*?\]", f"neighbors: {nb_str}", text, count=1, flags=re.MULTILINE)
    text = re.sub(r"^last_indexed:\s*\S+", f"last_indexed: {last_indexed}", text, count=1, flags=re.MULTILINE)

    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


# ---------------------------------------------------------------------------
# Heuristic dimension scorers
# ---------------------------------------------------------------------------

def _clamp(v: float) -> float:
    return max(0.0, min(1.0, v))


def score_specificity(content: str) -> float:
    """
    Measures concreteness. Signals:
    - Code blocks push toward 1.0
    - Numbers and measurements push up
    - Proper nouns (CamelCase, ALL_CAPS identifiers) push up
    - Short, vague content stays low
    """
    if not content:
        return 0.3

    score = 0.3  # baseline

    # Code blocks
    code_blocks = len(re.findall(r"```", content)) // 2
    score += min(0.3, code_blocks * 0.08)

    # Numbers and measurements
    numbers = len(re.findall(r"\b\d+(\.\d+)?(%|ms|px|kb|mb|gb|fps|hz)?\b", content))
    score += min(0.2, numbers * 0.015)

    # CamelCase / snake_case identifiers / SCREAMING_SNAKE (likely code/proper names)
    identifiers = len(re.findall(r"\b[A-Z][a-z]+[A-Z]\w*\b|\b[a-z]+_[a-z_]+\b|\b[A-Z]{2,}\b", content))
    score += min(0.2, identifiers * 0.01)

    # File references (file[NNNN]) — system-specific concrete references
    file_refs = len(re.findall(r"file\d{4}", content))
    score += min(0.1, file_refs * 0.02)

    # Markdown table rows — structured field/schema definitions are highly specific
    table_rows = len(re.findall(r"^\|[^|]+\|", content, re.MULTILINE))
    score += min(0.2, table_rows * 0.02)

    return _clamp(score)


def score_technicality(content: str) -> float:
    """
    Measures technical formality. Signals:
    - Code blocks are the strongest signal
    - Technical vocabulary
    - Presence of prose-only sections = lower
    """
    if not content:
        return 0.3

    score = 0.2  # baseline — slightly technical by default in this workspace

    # Code blocks are the strongest signal
    code_blocks = len(re.findall(r"```", content)) // 2
    score += min(0.4, code_blocks * 0.12)

    lower = content.lower()

    # Technical keywords
    tech_hits = sum(1 for kw in TECHNICAL_HIGH if kw in lower)
    score += min(0.3, tech_hits * 0.04)

    # Prose-only indicators reduce score
    prose_hits = sum(1 for kw in TECHNICAL_LOW if kw in lower)
    score -= min(0.15, prose_hits * 0.05)

    # JSON/YAML blocks
    structured = len(re.findall(r"\{[\s\S]{5,200}\}", content))
    score += min(0.1, structured * 0.03)

    return _clamp(score)


def score_temporality(content: str, created_date: str | None = None) -> float:
    """
    Measures time-sensitivity. 0=foundational, 1=ephemeral.
    """
    if not content:
        return 0.3

    score = 0.3  # baseline

    lower = content.lower()

    # Temporal high words
    temp_high_hits = sum(1 for kw in TEMPORAL_HIGH if kw in lower)
    score += min(0.35, temp_high_hits * 0.07)

    # Temporal low words
    temp_low_hits = sum(1 for kw in TEMPORAL_LOW if kw in lower)
    score -= min(0.2, temp_low_hits * 0.05)

    # Explicit dates in content (likely current/ephemeral)
    date_refs = len(re.findall(r"\b202\d-\d{2}-\d{2}\b", content))
    score += min(0.1, date_refs * 0.025)

    # File type signals from headers
    if re.search(r"^#.*(meeting|standup|status|daily|sprint)", content, re.MULTILINE | re.IGNORECASE):
        score += 0.2
    if re.search(r"^#.*(spec|architecture|design|principle|adr|contract)", content, re.MULTILINE | re.IGNORECASE):
        score -= 0.1

    return _clamp(score)


def score_centrality(content: str, data_dir: str, file_number: str) -> float:
    """
    Measures graph connectedness.
    - Outbound links to other files in content = base centrality
    - Ticker co-access count = learned centrality (improves over time)
    """
    score = 0.1  # baseline — isolated until proven otherwise

    # Count explicit file references in content
    outbound_refs = set(re.findall(r"file(\d{4})", content))
    outbound_refs.discard(file_number)  # don't count self-references
    score += min(0.4, len(outbound_refs) * 0.08)

    # Ticker analysis: how many other files appear in the same sessions as this file?
    ticker_path = os.path.join(data_dir, "ticker.log")
    if os.path.exists(ticker_path):
        co_access_sessions: set[str] = set()
        with open(ticker_path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                parts = line.split(" | ")
                if len(parts) >= 3 and parts[1].strip() == file_number:
                    co_access_sessions.add(parts[2].strip())

        # For each session this file was read in, count unique co-read files
        co_read_files: set[str] = set()
        if co_access_sessions:
            with open(ticker_path, encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line or line.startswith("#"):
                        continue
                    parts = line.split(" | ")
                    if len(parts) >= 3 and parts[2].strip() in co_access_sessions:
                        other = parts[1].strip()
                        if other != file_number:
                            co_read_files.add(other)

        score += min(0.4, len(co_read_files) * 0.04)

    return _clamp(score)


def score_confidence(content: str) -> float:
    """
    Measures certainty/establishment. 0=speculative, 1=verified.
    """
    if not content:
        return 0.5

    score = 0.5  # neutral baseline

    lower = content.lower()

    high_hits = sum(1 for kw in CONFIDENCE_HIGH if kw in lower)
    low_hits = sum(1 for kw in CONFIDENCE_LOW if kw in lower)
    med_hits = sum(1 for kw in CONFIDENCE_MED if kw in lower)

    score += min(0.35, high_hits * 0.07)
    score -= min(0.35, low_hits * 0.07)
    score += min(0.1, med_hits * 0.02)
    score -= min(0.1, med_hits * 0.02)  # net zero for medium signals

    # ADR status in content
    if re.search(r"status:\s*accepted", content, re.IGNORECASE):
        score += 0.15
    if re.search(r"status:\s*assumption", content, re.IGNORECASE):
        score -= 0.1
    if re.search(r"status:\s*proposed", content, re.IGNORECASE):
        score -= 0.15

    return _clamp(score)


# ---------------------------------------------------------------------------
# k-NN search
# ---------------------------------------------------------------------------

def _euclidean(a: list[float], b: list[float]) -> float:
    return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(5)))


def find_neighbors(
    data_dir: str,
    target_number: str,
    target_vector: list[float],
    k: int = 5,
) -> list[str]:
    """
    Find k nearest neighbors to target_vector among all indexed files in data_dir.
    Skips target_number itself. Skips deprecated files. Skips null-vector files.
    Returns list of file numbers sorted by ascending distance.
    """
    pattern = re.compile(r"^file(\d{4})\.md$")
    candidates: list[tuple[float, str]] = []

    for name in os.listdir(data_dir):
        m = pattern.match(name)
        if not m:
            continue
        number = m.group(1)
        if number == target_number:
            continue

        path = os.path.join(data_dir, name)
        try:
            meta, _ = _parse_file_for_indexing(path)
        except ValueError:
            continue

        if meta.get("deprecated"):
            continue

        raw_vec = meta.get("vector", [])
        if not raw_vec or any(v in (None, "null") for v in raw_vec):
            continue

        try:
            vec = [float(v) for v in raw_vec]
        except (ValueError, TypeError):
            continue

        dist = _euclidean(target_vector, vec)
        candidates.append((dist, number))

    candidates.sort(key=lambda x: x[0])
    return [number for _, number in candidates[:k]]


# ---------------------------------------------------------------------------
# Main index operation
# ---------------------------------------------------------------------------

def index_file(data_dir: str, file_number: str, k: int = 5) -> dict:
    """
    Compute 5D vector + k neighbors for file_number.
    Write results back into file metadata.
    Returns the result dict.
    """
    path = _file_path(data_dir, file_number)
    if not os.path.exists(path):
        return {"error": "file_not_found", "file_number": file_number}

    try:
        meta, content = _parse_file_for_indexing(path)
    except ValueError as e:
        return {"error": "parse_error", "file_number": file_number, "message": str(e)}

    if meta.get("deprecated"):
        return {"skipped": "deprecated", "file_number": file_number}

    created = meta.get("created")

    vector = [
        score_specificity(content),
        score_technicality(content),
        score_temporality(content, created_date=created),
        score_centrality(content, data_dir, file_number),
        score_confidence(content),
    ]

    neighbors = find_neighbors(data_dir, file_number, vector, k=k)

    last_indexed = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    _write_vector_back(path, vector, neighbors, last_indexed)

    return {
        "file_number": file_number,
        "vector": vector,
        "neighbors": neighbors,
        "last_indexed": last_indexed,
    }


def batch_index(data_dir: str, k: int = 5, force: bool = False) -> list[dict]:
    """
    Index all files with null vectors (or all files if force=True).
    Returns list of result dicts.
    """
    pattern = re.compile(r"^file(\d{4})\.md$")
    results = []

    candidates = []
    for name in sorted(os.listdir(data_dir)):
        m = pattern.match(name)
        if not m:
            continue
        number = m.group(1)
        path = os.path.join(data_dir, name)
        try:
            meta, _ = _parse_file_for_indexing(path)
        except ValueError:
            continue

        if meta.get("deprecated"):
            continue

        raw_vec = meta.get("vector", [])
        has_null = not raw_vec or any(v in (None, "null") for v in raw_vec)

        if force or has_null:
            candidates.append(number)

    for number in candidates:
        result = index_file(data_dir, number, k=k)
        results.append(result)
        if "error" in result:
            print(f"  ERROR {number}: {result.get('message', result.get('error'))}", file=sys.stderr)
        elif "skipped" in result:
            print(f"  SKIP  file{number} ({result['skipped']})")
        else:
            vec_str = "[" + ", ".join(f"{v:.3f}" for v in result["vector"]) + "]"
            nb_str = ", ".join(result["neighbors"]) if result["neighbors"] else "none"
            print(f"  file{number}  {vec_str}  neighbors: [{nb_str}]")

    return results


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="indexer — assign 5D vectors to knowledge graph files")
    sub = parser.add_subparsers(dest="command", required=True)

    p_index = sub.add_parser("index", help="Index one file")
    p_index.add_argument("data_dir")
    p_index.add_argument("file_number")
    p_index.add_argument("--k", type=int, default=5)

    p_batch = sub.add_parser("batch", help="Index all null-vector files")
    p_batch.add_argument("data_dir")
    p_batch.add_argument("--k", type=int, default=5)
    p_batch.add_argument("--force", action="store_true", help="Re-index even already-indexed files")

    p_reindex = sub.add_parser("reindex", help="Force re-index one file")
    p_reindex.add_argument("data_dir")
    p_reindex.add_argument("file_number")
    p_reindex.add_argument("--k", type=int, default=5)

    p_nb = sub.add_parser("neighbors", help="Print current neighbors of a file")
    p_nb.add_argument("data_dir")
    p_nb.add_argument("file_number")

    args = parser.parse_args()

    if args.command == "index":
        result = index_file(args.data_dir, args.file_number.zfill(4), k=args.k)
        if "error" in result:
            print(f"ERROR: {result}", file=sys.stderr)
            sys.exit(1)
        vec_str = "[" + ", ".join(f"{v:.3f}" for v in result["vector"]) + "]"
        print(f"file{result['file_number']}  vector: {vec_str}")
        print(f"neighbors: {result['neighbors']}")
        print(f"last_indexed: {result['last_indexed']}")

    elif args.command in ("batch", "reindex"):
        if args.command == "reindex":
            result = index_file(args.data_dir, args.file_number.zfill(4), k=args.k)
            print(result)
        else:
            results = batch_index(args.data_dir, k=args.k, force=args.force)
            ok = sum(1 for r in results if "vector" in r)
            print(f"\nIndexed {ok}/{len(results)} file(s).")

    elif args.command == "neighbors":
        path = _file_path(args.data_dir, args.file_number.zfill(4))
        if not os.path.exists(path):
            print(f"ERROR: file{args.file_number} not found", file=sys.stderr)
            sys.exit(1)
        meta, _ = _parse_file_for_indexing(path)
        print(f"file{args.file_number} neighbors: {meta.get('neighbors', [])}")


if __name__ == "__main__":
    main()
