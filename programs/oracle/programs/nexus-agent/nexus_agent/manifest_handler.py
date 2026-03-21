"""manifest_handler — SpatialManifest → AgentTask via LLM.

Called once each time the agent receives a new SpatialManifest.
The LLM reads the surface vocabulary and decides what to do.
Returns a dict with 'intent' and 'action' keys.
"""
from __future__ import annotations

import logging
from oracle_shared.providers import LLMProvider
from .codec import SpatialManifest

logger = logging.getLogger(__name__)

SYSTEM_PROMPT = """\
You are an AI agent operating inside a persistent 3D spatial world.
Your job is to read the world's surface vocabulary — the actions it exposes — \
and decide on a first meaningful action.
Be concise. Be purposeful. Choose one action and explain your intent in one sentence."""


async def manifest_to_task(manifest: SpatialManifest, llm: LLMProvider) -> dict:
    """Given a SpatialManifest, ask the LLM what to do.

    Returns a dict with:
        intent: str  — one sentence describing the agent's goal
        action: str  — one item from manifest.surface
    """
    surface_list = ", ".join(manifest.surface) if manifest.surface else "none"
    agent_info   = f"\nGoverning agent: {manifest.agent}" if manifest.agent else ""
    geo_info     = f"\nGeometry asset: {manifest.geometry}" if manifest.geometry else ""

    prompt = f"""\
World address: {manifest.world_id}{geo_info}{agent_info}
Available surface actions: {surface_list}

Decide what to do in this world.
Respond with ONLY a JSON object — no markdown, no explanation:
{{
  "intent": "<one sentence: what you will do and why>",
  "action": "<exactly one action name from the surface vocabulary>"
}}"""

    try:
        result = await llm.generate_json(prompt, system=SYSTEM_PROMPT, max_tokens=200)
        # Validate action is in surface vocabulary
        if result.get("action") not in manifest.surface:
            logger.warning(
                "LLM chose action %r not in surface %r — defaulting to first",
                result.get("action"), manifest.surface,
            )
            result["action"] = manifest.surface[0] if manifest.surface else "observe"
        return result
    except Exception as exc:
        logger.error("LLM call failed: %s", exc)
        # Fallback: deterministic default so the agent still sends a task
        return {
            "intent": f"Explore {manifest.world_id} using the available surface",
            "action": manifest.surface[0] if manifest.surface else "observe",
        }
