"""
context-builder — watches ticker.log, executes embedded prompts, writes ctx-NNNN.md files.

Implements ADR-005 (on-read trigger). Every time a file is read (new ticker entry),
context-builder generates or refreshes that file's context document.

The ctx file is a structured self-description: what the file is, where it sits in the
graph, how it relates to its neighbors, and what role it plays.

Note on "embedded prompt execution":
  In a live system, the embedded prompt is executed by Claude via the tool_use loop.
  This script implements a deterministic equivalent: it reads content + neighbors and
  generates the same structured output using heuristics. The format is identical.
  When the system is connected to the Claude API, this script is replaced by
  a Claude tool_use session — the ctx file format does not change.

Usage:
  python context_builder.py build <data_dir> <file_number> [--session S] [--staleness N]
      Build ctx for one file immediately. Respects staleness threshold.

  python context_builder.py watch <data_dir> [--interval 2] [--staleness 5] [--session S]
      Poll ticker.log. Build ctx for every new entry. Runs until Ctrl-C.

  python context_builder.py status <data_dir>
      Show which files have ctx files and how old they are.
"""

import argparse
import math
import os
import re
import sys
import time
from datetime import datetime, timezone


# ---------------------------------------------------------------------------
# Dimension interpretation
# ---------------------------------------------------------------------------

def _interp_specificity(v: float) -> tuple[str, str]:
    if v < 0.35:
        return "low", "abstract, conceptual content"
    if v < 0.65:
        return "medium", "mixed concrete and abstract"
    return "high", "specific, concrete, detailed"


def _interp_technicality(v: float) -> tuple[str, str]:
    if v < 0.35:
        return "low", "intuitive, accessible to non-technical readers"
    if v < 0.65:
        return "medium", "technical framing, some domain knowledge needed"
    return "high", "formal or code-level, technical audience"


def _interp_temporality(v: float) -> tuple[str, str]:
    if v < 0.35:
        return "stable", "foundational, unlikely to change"
    if v < 0.65:
        return "mixed", "moderate lifespan, may be revised"
    return "current", "time-sensitive, likely to change soon"


def _interp_centrality(v: float) -> tuple[str, str]:
    if v < 0.35:
        return "peripheral", "isolated, few connections to other documents"
    if v < 0.65:
        return "connected", "moderately linked to others"
    return "hub", "well-connected, many documents reference or co-occur with this one"


def _interp_confidence(v: float) -> tuple[str, str]:
    if v < 0.35:
        return "speculative", "hypothesis or early-stage thinking"
    if v < 0.65:
        return "probable", "likely correct but not verified"
    return "established", "confirmed, accepted, or implemented"


def interpret_vector(vector: list[float]) -> str:
    """Return the My Position block content for a given vector."""
    s_lbl, s_desc = _interp_specificity(vector[0])
    t_lbl, t_desc = _interp_technicality(vector[1])
    te_lbl, te_desc = _interp_temporality(vector[2])
    c_lbl, c_desc = _interp_centrality(vector[3])
    cf_lbl, cf_desc = _interp_confidence(vector[4])
    vec_str = "[" + ", ".join(f"{v:.3f}" for v in vector) + "]"

    return f"""\
Vector: {vec_str} — interpreted as:
- Specificity: {s_lbl} — {s_desc}
- Technicality: {t_lbl} — {t_desc}
- Temporality: {te_lbl} — {te_desc}
- Centrality: {c_lbl} — {c_desc}
- Confidence: {cf_lbl} — {cf_desc}"""


# ---------------------------------------------------------------------------
# File parser (minimal)
# ---------------------------------------------------------------------------

def _file_path(data_dir: str, number: str) -> str:
    return os.path.join(data_dir, f"file{number}.md")


def _ctx_path(data_dir: str, number: str) -> str:
    return os.path.join(data_dir, f"ctx-{number}.md")


def _parse_file(path: str, number: str) -> dict:
    """Parse a Data/ file. Returns dict with metadata, content, neighbors, vector."""
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
            if not inner:
                meta[key] = []
            else:
                items = [v.strip() for v in inner.split(",")]
                parsed = []
                for item in items:
                    if item == "null":
                        parsed.append(None)
                    else:
                        try:
                            parsed.append(float(item))
                        except ValueError:
                            parsed.append(item.strip('"'))
                meta[key] = parsed
        elif val == "null":
            meta[key] = None
        elif val in ("true", "false"):
            meta[key] = val == "true"
        else:
            try:
                meta[key] = int(val)
            except ValueError:
                meta[key] = val

    # Content (strip embedded prompt)
    body = raw[end_fence + 5:]
    content = re.sub(
        r"<!-- EMBEDDED PROMPT.*?<!-- END EMBEDDED PROMPT -->",
        "",
        body,
        flags=re.DOTALL,
    ).strip()

    return {
        "number": number,
        "vector": meta.get("vector", [None] * 5),
        "neighbors": [str(n).zfill(4) for n in meta.get("neighbors", []) if n is not None],
        "deprecated": meta.get("deprecated", False),
        "content": content,
        "raw": raw,
    }


# ---------------------------------------------------------------------------
# Ctx content generation
# ---------------------------------------------------------------------------

def _first_sentences(text: str, n: int = 3) -> str:
    """Extract first n sentences from text."""
    sentences = re.split(r"(?<=[.!?])\s+", text.strip())
    return " ".join(sentences[:n]).strip()


def _euclidean(a: list[float], b: list[float]) -> float:
    return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(5)))


def _infer_relationship(self_vec: list[float], neighbor_vec: list[float],
                        self_content: str, neighbor_content: str, neighbor_number: str) -> str:
    """
    Generate a one-sentence relationship description based on vector similarity
    and content signals.
    """
    if any(v is None for v in self_vec) or any(v is None for v in neighbor_vec):
        return f"file{neighbor_number} is a neighboring document (not yet indexed)."

    dist = _euclidean(self_vec, neighbor_vec)

    # Characterize the relationship by which dimension differs most
    diffs = [abs(self_vec[i] - neighbor_vec[i]) for i in range(5)]
    max_dim = diffs.index(max(diffs))
    dim_names = ["specificity", "technicality", "temporality", "centrality", "confidence"]

    if dist < 0.15:
        relationship = f"file{neighbor_number} is closely related (distance {dist:.3f}) — nearly identical position in the graph."
    elif max_dim == 0:
        if self_vec[0] > neighbor_vec[0]:
            relationship = f"file{neighbor_number} is a more abstract treatment of a similar topic (distance {dist:.3f})."
        else:
            relationship = f"file{neighbor_number} is a more concrete, detailed treatment of a similar topic (distance {dist:.3f})."
    elif max_dim == 1:
        if self_vec[1] > neighbor_vec[1]:
            relationship = f"file{neighbor_number} covers similar content from a less technical angle (distance {dist:.3f})."
        else:
            relationship = f"file{neighbor_number} is a more technical treatment of a related topic (distance {dist:.3f})."
    elif max_dim == 2:
        if self_vec[2] > neighbor_vec[2]:
            relationship = f"file{neighbor_number} is a more foundational, stable document on a related topic (distance {dist:.3f})."
        else:
            relationship = f"file{neighbor_number} is a more current, time-sensitive document on a related topic (distance {dist:.3f})."
    elif max_dim == 3:
        if self_vec[3] > neighbor_vec[3]:
            relationship = f"file{neighbor_number} is more isolated in the graph but covers related territory (distance {dist:.3f})."
        else:
            relationship = f"file{neighbor_number} is a better-connected hub document in a related cluster (distance {dist:.3f})."
    else:
        if self_vec[4] > neighbor_vec[4]:
            relationship = f"file{neighbor_number} is a less certain, more exploratory treatment of a related topic (distance {dist:.3f})."
        else:
            relationship = f"file{neighbor_number} is a more established, confirmed document in a related area (distance {dist:.3f})."

    return relationship


def _infer_cluster(vector: list[float], content: str) -> str:
    """Assign a cluster label based on vector position and content signals."""
    if any(v is None for v in vector):
        return "unindexed"

    content_lower = content.lower()

    # Content-based cluster signals (checked first — more precise)
    if re.search(r"\badr[-\s]?\d+\b|\barchitectural decision\b|\bstatus:\s*(accepted|proposed|assumption)\b",
                 content_lower):
        return "architectural decisions"
    if re.search(r"\bmeeting notes?\b|\bstandup\b|\bsprint\b|\bstatus update\b", content_lower):
        return "operational notes"
    if re.search(r"\bprd\b|\bproduct requirement\b|\buser stor\b|\bepic\b", content_lower):
        return "product requirements"
    if re.search(r"\bcontract\b|\bschema\b|\binterface\b|\bapi spec\b|\bdata shape\b", content_lower):
        return "contracts and interfaces"
    if re.search(r"\bimplementation\b|\bsrc\b|\bdef \b|\bclass \b|\balgorithm\b", content_lower):
        return "implementation specs"
    if re.search(r"\btest\b|\bvalidat\b|\bauditing\b|\bcheck\b", content_lower):
        return "testing and validation"
    if re.search(r"\broadmap\b|\bpriority\b|\bmilestone\b|\bphase\b", content_lower):
        return "planning and roadmap"

    # Fallback: vector-based cluster
    specificity, technicality, _, centrality, confidence = vector
    if centrality > 0.7:
        return "hub documents"
    if technicality > 0.6 and specificity > 0.6:
        return "technical implementation"
    if confidence < 0.4:
        return "open questions"
    if specificity < 0.3:
        return "conceptual foundations"
    return "general reference"


def _infer_role(vector: list[float], content: str, number: str) -> str:
    """Generate a 'This document is a X that Y' sentence."""
    if any(v is None for v in vector):
        return f'"This document is a file that has not yet been indexed or described."'

    content_lower = content.lower()
    specificity, technicality, temporality, centrality, confidence = vector

    # Noun
    if re.search(r"\badr[-\s]?\d+\b|\barchitectural decision\b", content_lower):
        noun = "decision record"
    elif re.search(r"\bmeeting notes?\b|\bstandup\b", content_lower):
        noun = "session log"
    elif re.search(r"\bprd\b|\bproduct requirement\b", content_lower):
        noun = "requirements document"
    elif re.search(r"\bcontract\b|\bschema\b|\bdata shape\b", content_lower):
        noun = "contract definition"
    elif re.search(r"\breadme\b|\boverview\b|\bintroduction\b", content_lower):
        noun = "orientation document"
    elif technicality > 0.6:
        noun = "technical specification"
    elif specificity < 0.3:
        noun = "conceptual document"
    else:
        noun = "reference document"

    # Verb phrase
    if centrality > 0.6:
        verb = "serves as a central hub connecting multiple areas of the knowledge base"
    elif temporality > 0.6:
        verb = "captures current state and is expected to be updated frequently"
    elif confidence > 0.7:
        verb = "records an established, accepted decision or fact"
    elif confidence < 0.4:
        verb = "explores an open question or speculative direction"
    elif technicality > 0.6:
        verb = "specifies implementation details for builders"
    else:
        verb = "provides reference information for navigating this area of the graph"

    return f'"This document is a {noun} that {verb}."'


def generate_ctx(
    data_dir: str,
    number: str,
    neighbor_records: list[dict],
    session_id: str = "context-builder",
) -> str:
    """
    Generate the full ctx-NNNN.md content for a file.
    neighbor_records: list of already-read file-record dicts for each neighbor.
    """
    path = _file_path(data_dir, number)
    record = _parse_file(path, number)

    vector = record["vector"]
    has_vector = not any(v is None for v in vector)
    content = record["content"]
    neighbors = record["neighbors"]

    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    vec_str = "[" + ", ".join(f"{v:.3f}" for v in vector) + "]" if has_vector else "[null, null, null, null, null]"

    # Header
    lines = [
        f"# Context — file{number}",
        "",
        f"Generated: {now}",
        f"Vector: {vec_str}",
        "",
    ]

    # What I Am
    lines += ["## What I Am", ""]
    summary = _first_sentences(content, n=3) if content else "(no content)"
    lines += [summary, ""]

    # My Position
    if has_vector:
        lines += ["## My Position", ""]
        lines += [interpret_vector(vector), ""]

    # My Neighbors
    lines += ["## My Neighbors and How I Relate to Them", ""]
    if not neighbors:
        lines += ["| Neighbor | Relationship |", "|----------|-------------|",
                  "| — | No neighbors indexed yet |", ""]
    else:
        lines += ["| Neighbor | Relationship |", "|----------|-------------|"]
        neighbor_map = {r["number"]: r for r in neighbor_records}
        for nb_num in neighbors:
            if nb_num in neighbor_map:
                nb = neighbor_map[nb_num]
                nb_vec = nb.get("vector", [None] * 5)
                # normalize: may come from file-selector (metadata.vector) or parse_file
                if isinstance(nb_vec, list) and len(nb_vec) == 5:
                    rel = _infer_relationship(
                        vector if has_vector else [None] * 5,
                        nb_vec,
                        content,
                        nb.get("content", ""),
                        nb_num,
                    )
                else:
                    rel = f"file{nb_num} is a neighboring document."
            else:
                rel = f"file{nb_num} is a neighboring document (not loaded)."
            lines.append(f"| file{nb_num} | {rel} |")
        lines.append("")

    # My Cluster
    cluster = _infer_cluster(vector if has_vector else [None] * 5, content)
    lines += ["## My Cluster", "", cluster, ""]

    # My Role
    role = _infer_role(vector if has_vector else [None] * 5, content, number)
    lines += ["## My Role", "", role, ""]

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Core build operation
# ---------------------------------------------------------------------------

def _ctx_age_minutes(data_dir: str, number: str) -> float | None:
    """Return age of ctx file in minutes, or None if it doesn't exist."""
    ctx = _ctx_path(data_dir, number)
    if not os.path.exists(ctx):
        return None
    age_sec = time.time() - os.path.getmtime(ctx)
    return age_sec / 60.0


def build_ctx(
    data_dir: str,
    file_number: str,
    session_id: str = "context-builder",
    staleness_minutes: float = 5.0,
) -> dict:
    """
    Build ctx-NNNN.md for file_number.
    Returns {"status": "written"|"skipped"|"error", "file_number": ..., ...}
    """
    # Staleness check
    age = _ctx_age_minutes(data_dir, file_number)
    if age is not None and age < staleness_minutes:
        return {
            "status": "skipped",
            "reason": f"ctx file is {age:.1f} min old (threshold: {staleness_minutes} min)",
            "file_number": file_number,
        }

    path = _file_path(data_dir, file_number)
    if not os.path.exists(path):
        return {"status": "error", "error": "file_not_found", "file_number": file_number}

    try:
        record = _parse_file(path, file_number)
    except ValueError as e:
        return {"status": "error", "error": str(e), "file_number": file_number}

    # Read neighbors
    neighbor_records = []
    for nb_num in record["neighbors"]:
        nb_path = _file_path(data_dir, nb_num)
        if os.path.exists(nb_path):
            try:
                nb_record = _parse_file(nb_path, nb_num)
                # Normalize vector to match file-selector shape
                nb_record["vector"] = nb_record.get("vector", [None] * 5)
                neighbor_records.append(nb_record)
            except ValueError:
                pass

    # Generate and write ctx file
    try:
        ctx_content = generate_ctx(data_dir, file_number, neighbor_records, session_id=session_id)
    except Exception as e:
        return {"status": "error", "error": f"generation failed: {e}", "file_number": file_number}

    ctx_path = _ctx_path(data_dir, file_number)
    with open(ctx_path, "w", encoding="utf-8") as f:
        f.write(ctx_content)

    # Append context_built to ticker
    ticker_path = os.path.join(data_dir, "ticker.log")
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with open(ticker_path, "a", encoding="utf-8") as f:
        f.write(f"{ts} | {file_number} | {session_id} | context_built\n")

    return {
        "status": "written",
        "file_number": file_number,
        "ctx_path": ctx_path,
    }


# ---------------------------------------------------------------------------
# Ticker watcher
# ---------------------------------------------------------------------------

SKIP_REASONS = {"context_built", "indexed"}


def watch(
    data_dir: str,
    interval_sec: float = 2.0,
    staleness_minutes: float = 5.0,
    session_id: str = "context-builder",
) -> None:
    """
    Poll ticker.log for new entries. For each new read event, build ctx.
    Runs until Ctrl-C.
    """
    ticker_path = os.path.join(data_dir, "ticker.log")
    if not os.path.exists(ticker_path):
        print(f"ERROR: ticker.log not found at {ticker_path}", file=sys.stderr)
        sys.exit(1)

    # Record current end of ticker so we only process new entries
    with open(ticker_path, encoding="utf-8") as f:
        f.seek(0, 2)
        position = f.tell()

    print(f"context-builder watching {ticker_path} (interval={interval_sec}s, staleness={staleness_minutes}min)")
    print("Press Ctrl-C to stop.")

    try:
        while True:
            time.sleep(interval_sec)
            with open(ticker_path, encoding="utf-8") as f:
                f.seek(position)
                new_lines = f.readlines()
                position = f.tell()

            for line in new_lines:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue

                parts = line.split(" | ")
                if len(parts) < 4:
                    continue

                file_number = parts[1].strip()
                reason = parts[3].strip()

                # Skip: these would cause infinite loops
                if reason in SKIP_REASONS:
                    continue

                print(f"  [{datetime.now(timezone.utc).strftime('%H:%M:%S')}] "
                      f"new read: file{file_number} ({reason})")

                result = build_ctx(
                    data_dir, file_number,
                    session_id=session_id,
                    staleness_minutes=staleness_minutes,
                )

                if result["status"] == "written":
                    print(f"    ✓ wrote ctx-{file_number}.md")
                elif result["status"] == "skipped":
                    print(f"    – skipped: {result['reason']}")
                else:
                    print(f"    ✗ error: {result.get('error')}", file=sys.stderr)

    except KeyboardInterrupt:
        print("\ncontext-builder stopped.")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="context-builder — generate ctx files from ticker events")
    sub = parser.add_subparsers(dest="command", required=True)

    # build
    p_build = sub.add_parser("build", help="Build ctx for one file immediately")
    p_build.add_argument("data_dir")
    p_build.add_argument("file_number")
    p_build.add_argument("--session", default="context-builder", dest="session_id")
    p_build.add_argument("--staleness", type=float, default=5.0,
                         help="Skip if ctx is newer than this many minutes (default 5)")
    p_build.add_argument("--force", action="store_true",
                         help="Ignore staleness threshold and always rebuild")

    # watch
    p_watch = sub.add_parser("watch", help="Watch ticker.log and build ctx on new reads")
    p_watch.add_argument("data_dir")
    p_watch.add_argument("--interval", type=float, default=2.0, dest="interval_sec")
    p_watch.add_argument("--staleness", type=float, default=5.0)
    p_watch.add_argument("--session", default="context-builder", dest="session_id")

    # status
    p_status = sub.add_parser("status", help="Show ctx file status for all files")
    p_status.add_argument("data_dir")

    args = parser.parse_args()

    if args.command == "build":
        staleness = 0.0 if args.force else args.staleness
        result = build_ctx(
            args.data_dir,
            args.file_number.zfill(4),
            session_id=args.session_id,
            staleness_minutes=staleness,
        )
        if result["status"] == "written":
            print(f"Written: {result['ctx_path']}")
        elif result["status"] == "skipped":
            print(f"Skipped: {result['reason']}")
        else:
            print(f"Error: {result.get('error')}", file=sys.stderr)
            sys.exit(1)

    elif args.command == "watch":
        watch(
            args.data_dir,
            interval_sec=args.interval_sec,
            staleness_minutes=args.staleness,
            session_id=args.session_id,
        )

    elif args.command == "status":
        pattern = re.compile(r"^file(\d{4})\.md$")
        print(f"{'File':<10} {'Has ctx':<10} {'Age (min)':<12}")
        print("-" * 35)
        for name in sorted(os.listdir(args.data_dir)):
            m = pattern.match(name)
            if not m:
                continue
            number = m.group(1)
            age = _ctx_age_minutes(args.data_dir, number)
            has = "yes" if age is not None else "no"
            age_str = f"{age:.1f}" if age is not None else "—"
            print(f"file{number}    {has:<10} {age_str}")


if __name__ == "__main__":
    main()
