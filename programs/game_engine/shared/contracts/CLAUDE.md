# CLAUDE.md — contracts

## What Is In This Directory

8 accepted subsystem interface contracts. These define the boundaries between server, client, and infrastructure layers.

| Contract | Defines | Consumed by |
|----------|---------|-------------|
| `world-state-contract.md` | object_record, entity_record, change_request, state queries | world/, engine/, agents |
| `simulation-contract.md` | run_tick signature, physics_body, tick_result, collision_data | world/simulation/ |
| `world-graph-contract.md` | world_record, portals, subworlds, constellation, traversal protocol | orchestration, engine/map |
| `node-registry-contract.md` | domain→process routing, node_descriptor, load metrics | world/node-manager/, gateway |
| `lod-system-contract.md` | LOD tier assignment, blend results, distance thresholds | engine/renderer/, engine/visibility/ |
| `player-session-contract.md` | auth token validation, session management | world/node-manager/, gateway |
| `ticker-log-contract.md` | audit log append, state change history | world/node-manager/ |
| `asset-store-contract.md` | asset caching, streaming, CDN delivery | engine/renderer/ |

## How these relate to each other

```
                    world-graph-contract
                    (world topology — which worlds exist, how they connect)
                            │
                    node-registry-contract
                    (within a world — which server process owns which spatial domain)
                            │
                    world-state-contract
                    (within a domain — what objects exist, their state)
                            │
              ┌─────────────┼─────────────┐
              │             │             │
    simulation-contract  ticker-log   player-session
    (tick physics)       (audit log)  (auth/identity)
              │
    lod-system-contract ← asset-store-contract
    (detail levels)       (geometry/texture delivery)
```

## Quick Rules

- Contracts are ACCEPTED and FINAL — do not modify without a new version number
- All contracts use the same data shape conventions (Vec3f64 for positions, uint64 for IDs)
- If code and contract disagree, fix the code
