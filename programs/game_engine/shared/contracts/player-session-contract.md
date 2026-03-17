---
name: player-session-contract
status: accepted
version: 0.1
published_by: authentication + session layer (above node-manager)
consumed_by: world/node-manager/, engine/ (client handshake)
---

# Player Session Contract

A player session is the authenticated binding between a human at a device and their avatar entity in the world. This contract defines what the session layer provides to nodes and clients.

## What This Contract Provides

**SESSION VALIDATION:**

`validate_token(auth_token)` → session_record | REJECTED
  - Validates a bearer token presented in HANDSHAKE
  - Returns the full session record on success
  - Returns REJECTED with reason on failure (expired, invalid, banned)
  - Latency target: < 20ms (this is on the critical path for new connections)

`get_session(player_id)` → session_record | NOT_FOUND
  - Returns current session state for a player
  - Used by nodes to look up a player when they appear in their domain

`update_last_position(player_id, position)` → void
  - Called by node when player disconnects or node shuts down gracefully
  - Persists the player's last known position so they re-enter at the right location

`heartbeat(player_id, node_id)` → void
  - Called by node each N seconds while player is connected
  - If node fails to heartbeat, session layer initiates reconnection for that player

**PLAYER INVENTORY:**

`get_inventory(player_id)` → inventory_snapshot
  - Full inventory state at last known sync point
  - Used on player login to restore their items

`commit_inventory_change(player_id, change_delta)` → accepted | conflict
  - Atomically applies an inventory change (add items, remove items)
  - Returns CONFLICT if items being removed are no longer present (race condition)

## session_record shape

```
SESSION_RECORD:
  player_id: uint64
  display_name: string
  auth_level: uint8 (player | moderator | admin)
  status: active | banned | suspended
  last_position: (float64, float64, float64)
  last_orientation: quaternion
  last_seen_node_id: uint64
  session_token_expires_at: timestamp
  gpu_caps: uint8 (bitmask: bit0=BC7, bit1=ASTC — see ADR-005)
  accumulated_play_time_seconds: uint64
  account_created_at: timestamp
```

## What This Contract Does NOT Provide

- Authentication credential storage (handled by auth service above this layer)
- Social features (friends, guilds) — those are game-logic layer
- Game state beyond position and inventory (that is the world graph)
- Movement validation (that is the simulation layer's job)
