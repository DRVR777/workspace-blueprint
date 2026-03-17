"""
Phase 0 stub: single node, hardcoded 1000×1000×1000 unit domain.

Per CONTEXT.md Step 1: "stub: hard-code a 1000×1000×1000 unit domain for Phase 0".
Interface matches node-registry-contract.md.
"""

from dataclasses import dataclass, field


@dataclass
class LoadMetrics:
    client_count: int         = 0
    entity_count: int         = 0
    tick_duration_ratio: float = 0.0


class NodeRegistryStub:
    NODE_ID    = 1
    DOMAIN_MIN = (0.0,    0.0,    0.0)
    DOMAIN_MAX = (1000.0, 1000.0, 1000.0)

    def __init__(self) -> None:
        self._registered = False
        self._load: LoadMetrics = LoadMetrics()

    # -- Reads -------------------------------------------------------------

    def get_domain(self) -> tuple[tuple, tuple]:
        return self.DOMAIN_MIN, self.DOMAIN_MAX

    def get_node_id(self) -> int:
        return self.NODE_ID

    # -- Writes (orchestration-only in production) -------------------------

    def register_node(self, node_id: int,
                      domain: tuple, address: str) -> str:
        self._registered = True
        return "accepted"

    def update_node_load(self, node_id: int,
                         metrics: LoadMetrics) -> str:
        self._load = metrics
        return "accepted"

    def deregister_node(self, node_id: int) -> str:
        self._registered = False
        return "accepted"
