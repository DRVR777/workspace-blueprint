"""
run_session.py — Claude API session runner for the knowledge-graph CDS.

Opens a live Claude API session with claude-opus-4-6, passes the file-selector
tool schema, and gives Claude a navigation goal. Claude traverses the document
graph using file_selector tool_use calls; this script executes each call via
handle_tool_call() and feeds the results back until stop_reason == "end_turn".

Usage:
    python run_session.py
    python run_session.py --goal "Find the core architectural decisions"
    python run_session.py --goal "Trace the 5D vector from spec to implementation"
    python run_session.py --start 0009 --goal "Explore neighbors of the 5D vector spec"

Requirements:
    pip install anthropic
    ANTHROPIC_API_KEY environment variable must be set.
"""

import argparse
import json
import os
import sys
from pathlib import Path

import anthropic

# ---------------------------------------------------------------------------
# Resolve paths
# ---------------------------------------------------------------------------

SCRIPT_DIR = Path(__file__).parent
DATA_DIR = SCRIPT_DIR / "Data"
FILE_SELECTOR_SRC = SCRIPT_DIR / "programs" / "file-selector" / "src"

# Display constants
PREVIEW_MAX_CHARS = 80

sys.path.insert(0, str(FILE_SELECTOR_SRC))
from file_selector import TOOL_SCHEMA, handle_tool_call  # noqa: E402

# ---------------------------------------------------------------------------
# Session runner
# ---------------------------------------------------------------------------

DEFAULT_GOAL = (
    "Explore the knowledge graph. Start at file 0006 (architecture overview), "
    "then follow its neighbors to understand the system. "
    "After reading at least 5 files, summarize what you learned about the "
    "Cognitive Document System and how its components relate to each other."
)

SYSTEM_PROMPT = """You are navigating a self-describing document graph called the
Cognitive Document System (CDS). Every document has a 5D position vector and a
pre-computed list of its nearest neighbors.

Use the file_selector tool to read documents and discover neighbors.
Each document you read has an embedded prompt at the top — follow its navigation
instructions. When a document tells you to read its neighbors, use the
query_vector or file_number parameters to do so.

Navigate until you have a clear picture of the topic, then synthesize what you
found into a coherent summary. Cite file numbers when referencing documents."""


def run_session(goal: str, start_file: str | None = None, session_id: str = "live") -> None:
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("ERROR: ANTHROPIC_API_KEY environment variable is not set.")
        sys.exit(1)

    if not DATA_DIR.exists():
        print(f"ERROR: Data directory not found: {DATA_DIR}")
        sys.exit(1)

    client = anthropic.Anthropic(api_key=api_key)

    # Build the opening user message
    if start_file:
        num = start_file.zfill(4)
        user_content = (
            f"Goal: {goal}\n\n"
            f"Start by reading file {num}."
        )
    else:
        user_content = f"Goal: {goal}"

    messages: list[dict] = [{"role": "user", "content": user_content}]

    print(f"\n{'='*70}")
    print("KNOWLEDGE GRAPH SESSION")
    print(f"Data directory : {DATA_DIR}")
    print(f"Goal           : {goal}")
    if start_file:
        print(f"Start file     : {start_file.zfill(4)}")
    print(f"{'='*70}\n")

    turn = 0
    tool_calls_total = 0

    # Agentic loop
    while True:
        turn += 1
        print(f"[turn {turn}] calling Claude...")

        response = client.messages.create(
            model="claude-opus-4-6",
            max_tokens=4096,
            system=SYSTEM_PROMPT,
            tools=[TOOL_SCHEMA],
            messages=messages,
        )

        # Append assistant response to history
        messages.append({"role": "assistant", "content": response.content})

        # Print any text blocks Claude produced
        for block in response.content:
            if block.type == "text" and block.text.strip():
                print(f"\n[Claude]\n{block.text}\n")

        # Check stop reason
        if response.stop_reason == "end_turn":
            print(f"\n{'='*70}")
            print(f"Session complete. Turns: {turn}  Tool calls: {tool_calls_total}")
            print(f"{'='*70}\n")
            break

        if response.stop_reason != "tool_use":
            print(f"Unexpected stop_reason: {response.stop_reason}. Ending session.")
            break

        # Execute tool calls
        tool_results = []
        for block in response.content:
            if block.type != "tool_use":
                continue

            tool_calls_total += 1
            tool_name = block.name
            tool_input = block.input

            print(f"  [tool_use #{tool_calls_total}] {tool_name}({_summarise_input(tool_input)})")

            if tool_name == "file_selector":
                result = handle_tool_call(
                    str(DATA_DIR),
                    {**tool_input, "session_id": session_id},
                )
                # Print brief result summary
                if "error" in result:
                    print(f"    → ERROR: {result['error']}: {result.get('message', '')}")
                elif "results" in result:
                    count = len(result["results"])
                    print(f"    → {count} file(s) returned from proximity query")
                else:
                    fn = result.get("file_number", "?")
                    preview = result.get("content", "")[:PREVIEW_MAX_CHARS].replace("\n", " ")
                    print(f"    → file{fn}: {preview}...")

                tool_results.append({
                    "type": "tool_result",
                    "tool_use_id": block.id,
                    "content": json.dumps(result),
                })
            else:
                # Unknown tool
                tool_results.append({
                    "type": "tool_result",
                    "tool_use_id": block.id,
                    "content": json.dumps({"error": "unknown_tool", "name": tool_name}),
                })

        # Feed results back
        messages.append({"role": "user", "content": tool_results})


def _summarise_input(inp: dict) -> str:
    if "file_number" in inp:
        return f"file_number={inp['file_number']}"
    if "query_vector" in inp:
        vec = inp["query_vector"]
        k = inp.get("k", 5)
        return f"query_vector={[round(v, 2) for v in vec]}, k={k}"
    return str(inp)[:60]


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(description="Run a Claude API session over the knowledge graph")
    parser.add_argument(
        "--goal",
        default=DEFAULT_GOAL,
        help="Navigation goal for Claude (default: architectural exploration)",
    )
    parser.add_argument(
        "--start",
        default=None,
        metavar="NNNN",
        help="File number to start from (e.g. 0009)",
    )
    parser.add_argument(
        "--session-id",
        default="live",
        help="Session ID written to ticker.log (default: live)",
    )
    args = parser.parse_args()

    run_session(
        goal=args.goal,
        start_file=args.start,
        session_id=args.session_id,
    )


if __name__ == "__main__":
    main()
