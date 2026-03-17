.PHONY: up down install install-shared logs \
        signal-ingestion whale-detector osint-fusion \
        solana-executor reasoning-engine knowledge-base \
        operator-dashboard

# ── Infrastructure ────────────────────────────────────────────────────────────

up:         ## Start Redis (required before running any program)
	docker compose up -d
	@echo "Redis running at localhost:6379"

down:       ## Stop Redis
	docker compose down

logs:       ## Tail Redis logs
	docker compose logs -f redis

# ── Install ───────────────────────────────────────────────────────────────────

install-shared:  ## Install oracle-shared package (run once, or after contract changes)
	cd oracle-shared && uv pip install -e .

install: install-shared  ## Install oracle-shared + all program dependencies
	@for prog in signal-ingestion whale-detector osint-fusion \
	             solana-executor reasoning-engine knowledge-base operator-dashboard; do \
	  echo "Installing $$prog..."; \
	  cd programs/$$prog && uv pip install -e . && cd ../..; \
	done

# ── Run programs ──────────────────────────────────────────────────────────────
# Each program reads .env automatically via python-dotenv.
# Start Redis first: make up

signal-ingestion:
	cd programs/signal-ingestion && uv run python -m signal_ingestion

whale-detector:
	cd programs/whale-detector && uv run python -m whale_detector

osint-fusion:
	cd programs/osint-fusion && uv run python -m osint_fusion

reasoning-engine:
	cd programs/reasoning-engine && uv run python -m reasoning_engine

knowledge-base:
	cd programs/knowledge-base && uv run python -m knowledge_base

solana-executor:
	cd programs/solana-executor && uv run python -m solana_executor

operator-dashboard:
	cd programs/operator-dashboard && uv run python -m operator_dashboard
	@echo "Dashboard at http://localhost:8080"

# ── Help ──────────────────────────────────────────────────────────────────────

help:       ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*##"}; {printf "  %-22s %s\n", $$1, $$2}'
