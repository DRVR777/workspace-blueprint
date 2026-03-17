---
name: ticker-log-contract
status: accepted
version: 0.1
published_by: ticker-log service (infrastructure)
consumed_by: world/node-manager/ (writes), analytics layer (reads), replay system (reads)
---

# Ticker Log Contract

The ticker log is the append-only history of the universe. Every state change that matters is recorded here. This contract defines how nodes write to the log and how consumers read it.

## What This Contract Provides

**WRITE (nodes call these):**

`append(entry)` → sequence_number
  - Appends one entry to the log
  - Returns the global monotonic sequence number assigned to this entry
  - Latency target: < 1ms (this is called in every tick for changed objects)
  - Durability guarantee: once append() returns, the entry will survive any single server failure

`append_batch(entries)` → list of sequence_numbers
  - Appends multiple entries atomically (all succeed or all fail)
  - More efficient than calling append() in a loop
  - Used at end of simulation tick to flush all changes at once

**READ (analytics, replay systems call these):**

`query_by_object(object_id, from_sequence, to_sequence)` → list of entries
  - All changes to a specific object in the sequence range
  - Used for object history, dispute resolution

`query_by_sector(sector_coords, from_time, to_time)` → list of entries
  - All changes to objects in a sector during the time range
  - Partitioned: only reads from the relevant time+sector partition

`query_by_sequence_range(from_sequence, to_sequence)` → list of entries
  - Raw sequential read — used for replay, compaction, auditing

`get_latest_sequence()` → uint64
  - Current global sequence number — used to determine how far behind a reader is

**SUBSCRIBE:**

`subscribe_to_sector(sector_coords)` → event stream of entries
  - Real-time stream of changes to objects in a sector
  - Used by adjacent nodes to observe events crossing their awareness boundary
  - Not used for within-domain changes (those come through the simulation result)

## Ticker Entry Shape

```
TICKER_ENTRY:
  sequence: uint64 — global monotonic, assigned by log service
  timestamp_us: uint64 — microseconds since epoch
  object_id: uint64
  sector_coords: (int32, int32, int32) — derived from object position at time of event
  event_type: uint16 — see event type registry
  source_type: uint8 — PLAYER=0, NODE=1, SYSTEM=2, AGENT=3
  source_id: uint64 — player_id or node_id
  session_id: uint64 — which server session this occurred in
  previous_state_hash: uint32 — quick check that previous state matches expectations
  payload_length: uint32
  payload: variable bytes — event-type-specific, compressed
```

## Event Type Registry (excerpt)

| Code | Event | Payload contains |
|------|-------|-----------------|
| 0x0001 | OBJECT_CREATED | full initial state |
| 0x0002 | OBJECT_DESTROYED | final state, cause |
| 0x0003 | POSITION_CHANGED | old_pos, new_pos |
| 0x0004 | PROPERTY_CHANGED | property_name, old_value, new_value |
| 0x0005 | OWNERSHIP_CHANGED | old_owner, new_owner |
| 0x0006 | INVENTORY_CHANGED | added_items, removed_items |
| 0x0007 | ENTITY_SPAWNED | entity_type, initial_state |
| 0x0008 | ENTITY_DESPAWNED | reason |
| 0x0009 | INTERACTION | interactor_id, interaction_type, result |
| 0x000A | DOMAIN_HANDOFF | object_id, from_node, to_node |

## Partition Strategy

The ticker log is partitioned by time-bucket × sector-cluster:
- Time bucket: 1 hour per bucket
- Sector cluster: 10×10×10 sectors per cluster (10,000 sectors per cluster)
- A query for "what happened in sector S during hour H" reads from exactly one partition
- Partitions older than RETENTION_PERIOD are archived to cold storage, then deleted
- RETENTION_PERIOD is a world configuration parameter (default: 90 days hot, 7 years cold)

## What This Contract Does NOT Provide

- Random write (append-only — no updates, no deletes)
- The ticker log is never the source of truth for current state — the world graph is
- Authentication of who is reading (the ticker log is internal infrastructure)
