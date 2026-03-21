"""nexus-agent configuration.

All values read from environment variables (via .env in oracle/ root).
Defaults allow offline/local development without a live server.
"""
import os
from dotenv import load_dotenv

load_dotenv()

# WebSocket URL of the NEXUS server
NEXUS_WS_URL: str = os.getenv("NEXUS_WS_URL", "ws://localhost:9001")

# World to enter on connect. Empty string = default world.
NEXUS_WORLD_ID: str = os.getenv("NEXUS_WORLD_ID", "")

# LLM provider (auto-detected from API keys if not set)
LLM_PROVIDER: str = os.getenv("LLM_PROVIDER", "")

# How long to wait (seconds) before re-sending a task after the world changes
TASK_COOLDOWN_S: float = float(os.getenv("NEXUS_AGENT_TASK_COOLDOWN_S", "30"))

# Reconnect delay on disconnect (seconds)
RECONNECT_DELAY_S: float = float(os.getenv("NEXUS_AGENT_RECONNECT_DELAY_S", "5"))
