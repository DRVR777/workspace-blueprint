#!/usr/bin/env bash
set -e

export PATH="$HOME/.local/bin:$PATH"
ORACLE="/mnt/c/Users/Quandale Dingle/yearTwo777/workspace-blueprint/workspace-blueprint/programs/oracle"

cd "$ORACLE"

echo "=== Creating venv ==="
uv venv --python 3.11 .venv
source .venv/bin/activate

echo "=== Installing oracle-shared ==="
uv pip install -e oracle-shared

echo "=== Installing signal-ingestion ==="
uv pip install -e "programs/signal-ingestion"

echo "=== Running test ==="
python "programs/signal-ingestion/tests/test_polymarket_rest.py"
