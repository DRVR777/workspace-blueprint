#!/usr/bin/env bash
set -e
export PATH="$HOME/.local/bin:$PATH"
ORACLE="/mnt/c/Users/Quandale Dingle/yearTwo777/workspace-blueprint/workspace-blueprint/programs/oracle"
cd "$ORACLE"
source .venv/bin/activate
python "programs/signal-ingestion/tests/test_polymarket_ws.py"
