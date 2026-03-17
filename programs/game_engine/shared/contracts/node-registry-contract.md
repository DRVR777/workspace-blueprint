---
name: node-registry-contract
status: accepted
version: 0.1
published_by: orchestration layer
consumed_by: world/node-manager/, engine/ (client routing), edge-gateway
---

# Node Registry Contract

The node registry is the authoritative map of which node owns which spatial domain at any given moment. It is the system that answers: "Given position P, which node should I talk to?"

## What This Contract Provides

**READ operations:**

`find_node(position)` → node_descriptor | NOT_COVERED
  - Given a 3D position, returns the node descriptor for the owning node
  - Returns NOT_COVERED if no node currently owns this position (will trigger spawn)
  - Read latency target: < 5ms

`get_all_nodes()` → list of node_descriptors
  - Returns all currently active nodes and their domains
  - Used by clients to build a local routing cache

`get_adjacent_nodes(node_id)` → list of node_descriptors
  - Returns all nodes whose domains share a boundary with the given node
  - Used by nodes to establish peer connections on startup

`get_node_status(node_id)` → node_status
  - Returns current status, load metric, client count, tick duration ratio

**WRITE operations (orchestration-only):**

`register_node(node_id, domain, address)` → accepted | conflict
  - Registers a new node. Fails if domain overlaps with existing registered domain.

`update_node_domain(node_id, new_domain)` → accepted | conflict
  - Updates domain after split/merge. Atomic — no transient overlap allowed.

`update_node_load(node_id, load_metrics)` → accepted
  - Nodes call this every N ticks to report current load. Used by orchestration.

`deregister_node(node_id)` → accepted
  - Marks a node as no longer active. Domain becomes NOT_COVERED until re-assigned.

## node_descriptor shape

```
NODE_DESCRIPTOR:
  node_id: uint64 — globally unique, never reused
  domain_min: (float64, float64, float64) — bounding box minimum corner
  domain_max: (float64, float64, float64) — bounding box maximum corner
  address: string — "hostname:port" for WebSocket connection
  status: active | draining | starting
  region_id: uint32 — which geographic region this node is in
  load_metrics:
    client_count: uint32
    entity_count: uint32
    tick_duration_ratio: float32  (actual_tick_ms / target_tick_ms; > 1.0 = overloaded)
    last_updated: timestamp
```

## Caching Rules

Clients and nodes may cache the domain map. Cache entries are valid for 30 seconds. After 30 seconds, re-query. If a routed connection is rejected (node says "this position is not in my domain"), invalidate cache and re-query immediately.

## Failure Behavior

If the node registry is unreachable:
- Nodes use their last known domain assignment
- Clients use cached routing
- New connections queue until registry recovers
- The node registry is a high-availability service — it must have no single point of failure
