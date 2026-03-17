---
name: node-manager
parent: world
type: sub-program
status: active
phase: 0
layer: 4 (world systems)
depends-on: spatial/
---

# node-manager — Node Lifecycle and Tick Orchestration

**What this sub-program is**: The top-level loop of the server-side game process. It owns the node's lifecycle (starting, active, draining, stopped) and runs the simulation tick by calling into simulation/ and spatial/. It is the entry point of the node process.

**What it owns**:
- Node startup: loading domain from orchestration, populating octree from world graph, opening WebSocket listener
- The simulation tick loop: the timed loop that calls simulation at TARGET_TICK_DURATION intervals
- Player connection management: accepting new WebSocket connections, routing actions to simulation
- State broadcasting: taking simulation results and sending the relevant subset to each connected client
- World graph write-back: queuing simulation results for async world graph persistence
- Inter-node messaging: sending and receiving peer messages (handoffs, boundary events)
- Domain split/merge orchestration: detecting overload, requesting split, executing split procedure
- Node shutdown: draining clients, flushing writes, reporting to orchestration

**What it does NOT own**:
- The physics math itself (that is simulation/)
- The spatial index (that is spatial/)
- The world graph database (external infrastructure)
- Client-side rendering (that is engine/)

**The tick loop specification**:

```
STARTUP:
  1. Read domain assignment from node-registry
  2. Connect to world graph (R-tree shard for this domain)
  3. Query world graph: all objects in domain → insert all into spatial index
  4. Query world graph: all entities in domain → add to active entity list
  5. Open WebSocket server on assigned port
  6. Register with node-registry: status = active
  7. Begin tick loop

TICK LOOP (runs every TARGET_TICK_DURATION ms, target = 20ms):
  tick_start = now()

  [Phase A] Collect this tick's inputs:
    actions = drain action_queue (all player inputs since last tick)
    peer_messages = drain peer_message_queue (from adjacent nodes)
    scheduled_events = pull any events scheduled for this tick

  [Phase B] Run simulation (call simulation/ contract):
    result = simulation.run_tick(
      world_snapshot: current objects + entities in domain,
      inputs: actions,
      messages: peer_messages,
      events: scheduled_events,
      dt: actual time since last tick (capped at MAX_TICK_DT = 50ms)
    )

  [Phase C] Apply results:
    FOR EACH change in result.state_changes:
      apply to local world snapshot
      enqueue for world graph write (async)
      enqueue for broadcasting

  [Phase D] Broadcast to clients:
    FOR EACH client C in connected_clients:
      relevant = filter changes by distance(change.position, C.player.position) <= C.visibility_radius
      send ENTITY_POSITION_UPDATE + OBJECT_STATE_CHANGE batch to C

  [Phase E] Flush ticker log:
    FOR EACH change in result.state_changes:
      ticker_log.append(change → ticker_entry)

  [Phase F] Self-monitor:
    tick_duration = now() - tick_start
    metrics.record(tick_duration)
    IF tick_duration > HIGH_LOAD_THRESHOLD:
      load_warning_count += 1
      IF load_warning_count > LOAD_GRACE_TICKS:
        request_split(orchestration)

  [Phase G] Sleep until next tick:
    sleep(max(0, TARGET_TICK_DURATION - tick_duration))

DRAIN (when orchestration requests shutdown):
  status = DRAINING
  stop accepting new client connections
  FOR EACH client C: initiate client handoff to adjacent node
  WAIT all handoffs complete
  FOR EACH entity E: write final state to world graph
  flush all pending world graph writes
  flush all pending ticker log writes
  deregister from node-registry
  EXIT
```

**Domain split procedure** (full spec):

```
SPLIT:
  adjacent_node = orchestration.spawn_node(half_of_my_domain)
  WAIT adjacent_node.status == STARTING

  split_boundary = midpoint of longest domain axis

  -- Transfer objects
  objects_to_transfer = spatial.query_box(adjacent_node.domain)
  FOR EACH O in objects_to_transfer:
    world_graph.update(O, domain_id = adjacent_node.id)
    spatial.remove(O.id)  -- remove from our octree

  -- Transfer entities / clients
  FOR EACH entity E with position in adjacent_node.domain:
    initiate_handoff(E, adjacent_node)

  -- Update our domain
  node_registry.update_node_domain(this.node_id, remaining_domain)
  adjacent_node.status = ACTIVE
```

**Performance targets** (Phase 0):
- Tick overhead (A + C + D phases, excluding simulation): < 3ms at 50 connected clients
- Total tick budget: 20ms (simulation gets 17ms)
- World graph write latency: < 10ms per object (async — does not block tick)
- Client broadcast throughput: 1,000 position updates encoded and sent per 20ms tick
