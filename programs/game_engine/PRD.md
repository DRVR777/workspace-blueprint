# NEXUS GAME ENGINE
## Product Requirements Document
### Version 0.1 — DRAFT | Status: scaffold | Date: 2026-03-13

---

> **GOVERNING RULE**: Until this PRD reaches status `complete`, no implementation code is written anywhere in this project. Every section uses pseudocode — structured behavioral description — not executable syntax. Language choices, framework selections, and library decisions are deferred until all sections of this document pass spec review.

---

## DOCUMENT MAP

| Part | Topic | Status |
|------|-------|--------|
| Part I | Vision & Anti-Patterns | DRAFT |
| Part II | Core Concepts & Glossary | DRAFT |
| Part III | World Architecture | DRAFT |
| Part IV | Node System | DRAFT |
| Part V | Local Engine | DRAFT |
| Part VI | LOD System | DRAFT |
| Part VII | Asset Pipeline | DRAFT |
| Part VIII | Network Protocol | DRAFT |
| Part IX | World Graph & Object Database | DRAFT |
| Part X | Simulation System | DRAFT |
| Part XI | Player System | DRAFT |
| Part XII | Agent System | DRAFT |
| Part XIII | World Economy & Building | DRAFT |
| Part XIV | Platform & Orchestration | DRAFT |
| Part XV | Modularity Specification | DRAFT |
| Part XVI | Performance Contracts | DRAFT |
| Part XVII | Implementation Phases | DRAFT |
| Appendix A | ELEV8 Failure Analysis | DRAFT |
| Appendix B | Knowledge-Graph Concepts Applied | DRAFT |
| Appendix C | Open Architectural Decisions | DRAFT |
| Appendix D | Dreamworld Requirements Map | DRAFT |

---

# PART I: VISION AND ANTI-PATTERNS

## 1.1 What We Are Building

NEXUS is a spatial computing substrate — a game engine purpose-built for one category of experience: an infinite, persistent, massively multiplayer three-dimensional universe in which every object is simulated, every player is spatially present, and the world continues to exist and change whether or not any player is watching.

This is not a general-purpose game engine. It does not need to run a racing game or a card game or a platformer. It needs to run one thing extremely well: a world. A world that is:

- **Infinite in extent**: A player should never encounter a boundary, a wall, or an edge. The universe extends in all three dimensions without limit. In practice, the universe is as large as the hardware that sustains it, and that hardware can be expanded at runtime without player disruption.

- **Persistent in state**: An object placed in the world by a player at 3am on a Tuesday exists when that player returns on Saturday. It has aged. Other players may have interacted with it. The world does not reset. It does not roll back. It accumulates history.

- **Massively shared**: Millions of players share one world. They are not in separate instances or shards. There is one world, and every player is in it simultaneously. Their actions affect each other.

- **Physically simulated**: The world has physics. Objects fall under gravity. Collisions have consequences. Energy is conserved. The simulation is not perfect real-world physics, but it is consistent and non-arbitrary. Players learn the rules and trust them.

- **Agent-inhabited**: Non-player entities inhabit the world alongside players. These agents have goals, perceptions, and behaviors. They are not scripted sequences — they are entities with embedded instructions that respond to their environment.

- **Player-buildable**: Players can change the world. They can place, modify, destroy, and create objects. They can build functional machines. The things they build persist and affect others.

## 1.2 Why Existing Approaches Fail At This

Traditional game engines — even the best commercial ones — are built around a different model. They assume:
- A finite, authored world (designed by a level designer, bounded by invisible walls)
- A bounded player count (typically single player or up to ~100 players)
- A single server (or a small fixed cluster) running the authoritative simulation
- Assets loaded at the start of a level (a "loading screen" is acceptable)
- State that resets between sessions (the game "reloads")

When you try to build an infinite, persistent, million-player world on top of these assumptions, every assumption fights you:
- The finite world assumption means the engine does not understand dynamic spatial partitioning
- The bounded player count assumption means the networking layer was never designed for this load
- The single server assumption means there is no protocol for cross-node entity handoff
- The loading screen assumption means there is no streaming architecture for continuous asset delivery
- The resetting state assumption means there is no durability layer for persistent world state

NEXUS is built without these assumptions. Every design decision starts from the infinite, persistent, massive-scale requirements and works backward to the implementation.

## 1.3 The ELEV8 Anti-Patterns (What This Engine Explicitly Avoids)

*Full analysis in Appendix A. Summary here for reference in all subsequent design decisions.*

**Anti-Pattern 1: Fragmented Stack**
ELEV8 ran three separate codebases (museum viewer, graph visualizer, AI chatbot) behind a single proxy. These three systems had no shared data model, no shared state, and no shared protocol. When one system's state needed to be visible in another, it had to be serialized, transmitted, and deserialized — across a process boundary — every time.

NEXUS constraint: One data model. One protocol. One state representation. All subsystems read from and write to the same world model. There is no translation layer between subsystems.

**Anti-Pattern 2: Physics Reinvention**
ELEV8 spent approximately 20 hours of development time building a custom force-directed 3D graph physics engine from scratch. This engine worked but was mathematically fragile, computationally expensive, and lacked the breadth of a proven implementation.

NEXUS constraint: The physics model is specified abstractly first (in this document). The specification defines behaviors, not implementations. The implementation uses the most appropriate proven data structures and algorithms. No physics logic is built from scratch if a well-understood algorithm exists.

**Anti-Pattern 3: Untyped State Sync**
ELEV8's WebSocket layer sent and received arbitrary JSON. When a message was malformed — due to a LLM hallucination, a network error, or a race condition — the system failed silently. There was no recovery path, no acknowledgment system, no schema validation.

NEXUS constraint: All messages over the network have a defined schema. Every message type has a version. The receiver validates every incoming message. Malformed messages trigger a defined error recovery path. Critical state changes require acknowledgment.

**Anti-Pattern 4: No Authoritative State**
ELEV8's rendering state and database state diverged. Three.js objects were mutated directly (`.position.set()`), bypassing the React state that the database layer expected to be the source of truth. This created invisible inconsistencies that were nearly impossible to debug.

NEXUS constraint: The world graph database is the single source of truth. The local rendering engine is a consumer of that truth. It never mutates the truth directly — it submits change requests that flow through the simulation layer and return as authoritative state updates.

**Anti-Pattern 5: Spatial Data Without Spatial Indexes**
ELEV8 stored positions in a Postgres table with no spatial indexing. Queries like "find all objects within 100 units of position X" required a full table scan.

NEXUS constraint: The world graph database has spatial indexes as a first-class requirement. Every object's position is indexed. Spatial queries are answered in logarithmic time, not linear time. The index is the most important part of the schema, not an optimization added later.

**Anti-Pattern 6: Deployment Coupling**
ELEV8's three subsystems were all deployed together as one unit. Deploying a change to the AI chatbot required restarting the museum viewer. Scaling the physics engine also scaled the chatbot.

NEXUS constraint: Every subsystem is independently deployable. A change to the LOD system does not require restarting the asset streaming service. Scaling the node cluster does not require touching the client renderer.

---

# PART II: CORE CONCEPTS AND GLOSSARY

## 2.1 Fundamental Terms

These terms have precise meanings throughout this document. When a term is used, it means exactly what this section says it means, not a colloquial interpretation.

**Universe**
The totality of simulated three-dimensional space. The universe has no boundaries. It extends in all three spatial dimensions without limit. Anything that "exists in the game" exists in the universe at some three-dimensional coordinate.

**World**
The subset of the universe that currently has content — objects, terrain, entities, events. The universe is infinite; the world is finite but growing. The world's boundary is not a wall — it is simply the edge of where content has been generated or placed. Beyond the world's current edge, the universe is empty and unrendered.

**Position**
A triple of real numbers (X, Y, Z) describing a location in the universe. All objects, entities, players, and nodes have a position or a bounded region defined by positions. Positions are continuous — there is no grid, no tile system, no discrete step.

**Domain**
A three-dimensional region of the universe owned by exactly one node at any time. A domain is defined by its bounding volume. Domains tile the occupied universe with no gaps and no overlaps.

**Node**
A running process on a server that owns a domain. A node:
- Simulates everything within its domain each tick
- Maintains connections to the players within its domain
- Writes world state changes to the world graph
- Communicates with adjacent nodes about entities crossing boundaries
- Serves assets to its connected players

A node is the fundamental unit of server-side computation. The node IS the game server — not a monolithic single server, but one of potentially thousands.

**Tick**
A fixed unit of simulation time. Every tick, every node computes the new state of everything in its domain. A tick has a target duration (e.g., 20ms for 50 simulations per second). If a tick takes longer than its target duration, the node is overloaded and the orchestration system may subdivide its domain.

**Client**
The local machine running the player's view of the game. The client renders the world, handles player input, runs local physics prediction, manages the asset cache, and maintains a WebSocket connection to the node whose domain contains the player.

**World Graph**
The distributed database that stores the state of every object in the universe. It is the single source of truth. Nodes read from it to load domain state on startup. Nodes write to it when state changes. The world graph is not owned by any single node — it is shared infrastructure.

**Object**
Anything that exists in the world as a discrete entity: a rock, a tree, a building, a player's body, an AI agent, a machine, a projectile, a particle emitter, a terrain chunk. Everything is an object. Objects have records in the world graph.

**Entity**
An object that can act — it has a simulation loop that runs each tick. Players and AI agents are entities. Static rocks and terrain are objects but not entities (they can be changed, but they don't act on their own).

**Asset**
The geometric description of an object type — its shape, its surface properties, its animation data. Assets are stored in the asset store. Clients download assets on demand and cache them locally. Assets are separate from object state — the shape of a tree type is an asset; the position of a specific tree is object state in the world graph.

**LOD (Level of Detail)**
A system for rendering objects at different geometric complexities based on their distance from the player. A rock has multiple LOD tiers: full detail (1,000 polygons at 5 meters), medium detail (200 polygons at 50 meters), low detail (20 polygons at 200 meters), impostor (single flat image at 500 meters), invisible (beyond 1,000 meters). The LOD system selects the appropriate tier per object per frame.

**Metadata**
The continuously streamed per-tick information about objects: their current positions, states, events. Metadata is not geometry — it is facts about where things are and what state they are in. Metadata is small, frequent, and authoritative.

**Spatial Partitioning**
The practice of dividing three-dimensional space into regions to accelerate spatial queries. Instead of checking every object when you want to know "what is within 100 units of position X," you check only the objects in the relevant spatial partition. NEXUS uses an octree as its primary spatial partitioning structure.

**Octree**
A tree data structure where each node represents a cubic volume of space. If a cube contains more than a threshold number of objects, it subdivides into 8 equal sub-cubes (hence "octo"). This allows spatial queries to skip entire regions of space that cannot contain relevant objects. An octree query for "what is within radius R of position P" is O(log N + K) where N is total objects and K is result count, instead of O(N).

**BVH (Bounding Volume Hierarchy)**
A tree structure for efficient collision detection. Each node in the tree is a bounding box that contains all the geometry of its subtree. Collision tests walk the tree, pruning branches whose bounding boxes do not intersect the test volume.

**Handoff**
The protocol by which a player or other entity transitions from one node's domain to another's. The entity's connection migrates from the old node to the new node. Both nodes participate. The entity never experiences a discontinuity.

**Ticker Log**
An append-only log of every state change in the world. Each entry records: timestamp, object identity, what changed, who caused it, previous value. The ticker log is the world's history. It enables replay, audit, and conflict resolution.

**Impostor**
The lowest LOD tier for an object — a flat image (billboard) that always faces the camera. From a distance, an impostor is visually indistinguishable from a fully rendered object at that scale. An impostor costs almost nothing to render.

**World Seed**
A deterministic value used to procedurally generate terrain. Given the same seed and the same position, the terrain generator always produces the same terrain. This means terrain does not need to be stored in the world graph — it can be regenerated on demand. Only modifications to the procedural terrain (a hole dug, a structure built) need to be stored.

## 2.2 System Relationships

```
UNIVERSE
  └── WORLD (the populated region of the universe)
        ├── DOMAINS (spatial regions, one per node)
        │     └── NODE (process owning the domain)
        │           ├── ENTITIES (things that act)
        │           └── OBJECTS (things that exist)
        └── WORLD GRAPH (shared truth across all domains)
              ├── OBJECT RECORDS (state of every object)
              ├── ASSET STORE (geometry of every object type)
              └── TICKER LOG (history of every change)

CLIENT
  ├── RENDERER (draws the world)
  ├── LOD SYSTEM (selects detail levels)
  ├── ASSET CACHE (locally stored geometry)
  ├── LOCAL PHYSICS (prediction, corrected by server)
  └── WEBSOCKET (connection to owning node)
```

---

# PART III: WORLD ARCHITECTURE

## 3.1 The Spatial Partitioning Hierarchy

The universe is partitioned spatially using a three-level hierarchy:

**Level 1: Sectors**
The universe is divided into large cubic sectors, each 1,000 units on a side (the exact size is an open ADR — see Appendix C, ADR-001). A sector is the unit of world-graph partitioning. All objects within a sector are stored in the same world-graph shard. Sectors are identified by their integer grid coordinates (sector_x, sector_y, sector_z).

A sector exists as a database entity even if it contains no objects. Its existence signals to the orchestration system that this region of space is "known." An unknown sector — one that has never been explored — is not in the database at all. When a player first enters an unknown sector, the terrain generator generates it on demand, and the sector record is created.

**Level 2: Chunks**
Each sector is subdivided into chunks, each 100 units on a side (so 10×10×10 = 1,000 chunks per sector). A chunk is the unit of terrain storage. When terrain is modified, only the affected chunk's modification record needs to be written. Unmodified chunks are regenerated from the world seed when needed.

A chunk is also the minimum spatial unit for LOD streaming. The client requests terrain data one chunk at a time. Adjacent chunks load as the player approaches their boundary.

**Level 3: Objects**
Within chunks, individual objects are stored with exact floating-point positions. The octree subdivision within a chunk allows efficient per-object queries.

## 3.2 The Octree

The octree is the primary spatial index. It exists at two levels:
- **World Graph Octree**: The global octree across all sectors, used for cross-domain queries (e.g., "find the nearest node to this position")
- **Node Octree**: Each node maintains a local octree of all objects in its domain, used for per-tick simulation (e.g., "which objects are within 10 units of this entity")

The node octree is rebuilt from the world graph when a node starts. It is maintained in memory during the node's lifetime. Changes are written to the world graph asynchronously.

### Octree Invariants

These invariants must always hold:
1. Every object in the universe is contained in exactly one leaf node of the world graph octree
2. Every object in a node's domain is contained in that node's local octree
3. The node octree is always consistent with the world graph — if they diverge, the world graph is authoritative

### Octree Operations (Pseudocode Specification)

**INSERT object at position P:**
```
FIND the leaf node L of the octree that contains P
ADD object to L's object list
IF L's object count exceeds MAX_OBJECTS_PER_LEAF:
  SUBDIVIDE L into 8 child nodes
  REDISTRIBUTE L's objects into the appropriate child nodes
  MARK L as an internal node (it no longer holds objects directly)
WRITE the updated octree structure to the world graph
```

**QUERY all objects within sphere(center C, radius R):**
```
BEGIN at the root of the octree
FOR EACH node N in breadth-first order:
  IF N's bounding box does not intersect sphere(C, R):
    SKIP N and all its descendants
  ELSE IF N is a leaf node:
    FOR EACH object O in N:
      IF distance(O.position, C) <= R:
        ADD O to result set
  ELSE:
    RECURSE into N's 8 children
RETURN result set
```

**REMOVE object from position P:**
```
FIND the leaf node L of the octree that contains P
REMOVE object from L's object list
IF L's object count falls below MIN_OBJECTS_PER_LEAF
AND L's parent has fewer than MERGE_THRESHOLD objects across all 8 children:
  MERGE the 8 children back into the parent
  MARK parent as a leaf node again
WRITE the updated octree structure to the world graph
```

**MOVE object from position P1 to P2:**
```
IF P1 and P2 are in the same leaf node:
  UPDATE the object's position in place
  (no octree restructuring needed)
ELSE:
  REMOVE object from P1's leaf node (may trigger merge)
  INSERT object at P2 (may trigger subdivision)
WRITE position change to the world graph ticker log
```

## 3.3 Domain Boundaries and Node Assignment

The domain assignment system maps positions to nodes. It answers the question: "Which node owns position P?"

The domain map is stored in the orchestration database — a separate, high-availability data store that all nodes and clients can query. The domain map is a spatial index of domain boundaries to node addresses.

### Domain Map Structure

```
FOR EACH active node N:
  N.domain = bounding box (min_x, min_y, min_z, max_x, max_y, max_z)
  N.address = network address:port where N is reachable
  N.status = active | draining | starting
  N.load = current load metric (entities per tick, tick duration ratio)
```

### Domain Lookup (Pseudocode)

```
FUNCTION find_node_for_position(P):
  QUERY domain map for node N where N.domain contains P
  IF no node found:
    -- This position is in a known but uncovered region
    TRIGGER domain_spawn event in orchestration layer
    WAIT for new node to come online
    RETRY find_node_for_position(P)
  RETURN N
```

## 3.4 World Generation

The world is generated procedurally. The procedural generation system takes a position (sector coordinates) and returns a complete terrain description for that sector — heights, biomes, natural features, resource deposits.

The generator is deterministic: same position, same output, always. This means:
- Terrain does not need to be stored before a player visits it
- Any node can generate terrain for any sector independently
- Two nodes generating the same sector at the same time produce identical output

**Generator Layers (Pseudocode):**

```
FUNCTION generate_sector(sector_x, sector_y, sector_z, world_seed):

  -- Layer 1: Base heightmap
  height_map = SAMPLE noise_function(
    frequency=BASE_FREQUENCY,
    octaves=8,
    seed=HASH(world_seed, sector_x, sector_z)
  )

  -- Layer 2: Biome assignment
  temperature_map = SAMPLE temperature_noise(sector_x, sector_z, world_seed)
  humidity_map = SAMPLE humidity_noise(sector_x, sector_z, world_seed)
  biome_map = LOOKUP biome_table[temperature, humidity]

  -- Layer 3: Feature placement
  FOR EACH feature type (trees, rocks, water, caves):
    feature_positions = SCATTER feature_type using
      SAMPLE feature_density_noise(sector_x, sector_z, world_seed, feature_type)
    FOR EACH position P in feature_positions:
      ADD feature_object to sector at P

  -- Layer 4: Resource placement
  FOR EACH resource type:
    resource_veins = PLACE using geological_noise(sector, world_seed, resource_type)
    FOR EACH vein V:
      ADD resource_deposit to sector at V.position with quantity V.amount

  -- Layer 5: Load sector modifications from world graph
  modifications = QUERY world_graph for modifications in this sector
  FOR EACH modification M:
    APPLY M to the generated sector
    -- (player-dug holes, placed objects, destroyed features, etc.)

  RETURN assembled sector
```

## 3.5 The World Boundary Protocol

When a player or entity approaches a domain boundary:

```
THRESHOLD_DISTANCE = 50 units from domain edge

WHEN entity E is within THRESHOLD_DISTANCE of domain edge:

  -- Current node initiates handoff
  adjacent_node = find_node_for_position(E.position + direction_of_approach)

  SEND handoff_prepare to adjacent_node:
    entity_id: E.id
    entity_state: E.current_state
    estimated_arrival_time: computed from E.velocity

  adjacent_node RESPONDS:
    status: ready
    accepted: true

  -- Client is informed
  SEND to_client:
    type: node_transfer_pending
    new_node_address: adjacent_node.address
    transfer_token: GENERATE secure_token(E.id, adjacent_node.id)

  -- Client connects to new node (maintains old connection)
  CLIENT connects to adjacent_node using transfer_token
  adjacent_node VALIDATES token and begins accepting entity state

  -- At the exact boundary crossing:
  old_node SENDS to adjacent_node:
    type: handoff_complete
    final_entity_state: E.current_state

  adjacent_node ACKNOWLEDGES
  old_node DROPS its reference to E
  client CLOSES connection to old_node

  -- E is now fully in adjacent_node's domain
```

---

# PART IV: NODE SYSTEM

## 4.1 What A Node Is

A node is the fundamental unit of server-side computation. It is a process — not a machine, not a container, not a database. A process, running on some hardware, responsible for simulating a defined region of the universe.

A node is stateless between restarts with respect to world data — all world state is in the world graph database. A node can crash and restart, loading its domain from the world graph, and the world continues without loss (with a brief gap in simulation while the node restarts, covered by the adjacent nodes' boundary handling).

A node has exactly one job: simulate its domain accurately, efficiently, and durably each tick.

## 4.2 The Node Lifecycle

```
STATE MACHINE: Node Lifecycle

  [STARTING]
    -- Node process launches
    -- Reads domain assignment from orchestration
    -- Connects to world graph
    -- Loads octree for domain from world graph
    -- Loads all entities in domain from world graph
    -- Opens WebSocket listener for clients
    -- Reports status = active to orchestration
    TRANSITION TO [ACTIVE]

  [ACTIVE]
    -- Normal operation: run simulation ticks
    -- Accept client connections
    -- Process player actions
    -- Communicate with adjacent nodes
    -- Monitor own load metrics
    IF load_metric > HIGH_LOAD_THRESHOLD:
      REQUEST domain_split from orchestration
    IF load_metric < LOW_LOAD_THRESHOLD for IDLE_DURATION:
      REQUEST domain_merge from orchestration
    TRANSITION TO [DRAINING] if orchestration requests shutdown

  [DRAINING]
    -- Orchestration has requested this node shut down
    -- Stop accepting new client connections
    -- Initiate handoff of all current clients to adjacent nodes
    -- Flush all pending world graph writes
    -- Wait for all handoffs to complete
    TRANSITION TO [STOPPED]

  [STOPPED]
    -- Process exits cleanly
    -- Orchestration marks domain as unassigned
    -- Adjacent nodes or new nodes pick up domain coverage
```

## 4.3 The Simulation Tick

The simulation tick is the heartbeat of the node. Every tick:

```
FUNCTION run_tick(node, tick_number):

  tick_start = now()

  -- Phase 1: Collect inputs
  player_actions = DRAIN action_queue  -- all player inputs since last tick
  network_messages = DRAIN message_queue  -- messages from adjacent nodes

  -- Phase 2: Apply player actions
  FOR EACH action A in player_actions:
    result = apply_action(A, node.world_state)
    IF result.valid:
      node.world_state = result.new_state
      pending_writes.ADD(result.state_change)
      pending_broadcasts.ADD(result.state_change)
    ELSE:
      SEND rejection_message to A.source_client

  -- Phase 3: Run entity behaviors
  FOR EACH entity E in node.active_entities:
    E.behavior.tick(E, node.world_state, tick_number)
    IF E.state_changed:
      pending_writes.ADD(E.state_change)
      pending_broadcasts.ADD(E.state_change)

  -- Phase 4: Run physics
  FOR EACH physics_body B in node.simulation:
    integrate_forces(B, TICK_DURATION)
    detect_and_resolve_collisions(B, node.octree)
    IF B.position_changed significantly:
      pending_writes.ADD(B.state_change)
      pending_broadcasts.ADD(B.state_change)

  -- Phase 5: Process network messages from adjacent nodes
  FOR EACH message M in network_messages:
    handle_inter_node_message(M, node)

  -- Phase 6: Broadcast state changes to clients
  FOR EACH client C connected to this node:
    relevant_changes = FILTER pending_broadcasts WHERE
      distance(change.position, C.player.position) <= C.visibility_radius
    SEND batch_update to C containing relevant_changes

  -- Phase 7: Persist to world graph (async, batched)
  ENQUEUE pending_writes to world_graph_write_buffer

  tick_duration = now() - tick_start
  metrics.record_tick_duration(tick_duration)

  IF tick_duration > TARGET_TICK_DURATION:
    metrics.record_overrun(tick_duration - TARGET_TICK_DURATION)
```

## 4.4 Node-to-Node Communication

Nodes communicate through two channels:

**Channel 1: Direct peer messaging**
For time-sensitive communication between adjacent nodes — entity handoffs, physics constraint resolution across boundaries, event propagation. These messages are sent directly between node processes, not through the world graph.

**Channel 2: World graph**
For durable state synchronization. Everything that must survive a node restart goes through the world graph. Adjacent nodes that need to know about state changes read from the world graph.

**Message Types (Peer Channel):**

```
HANDOFF_PREPARE
  sender: node_id
  entity_id: id
  entity_state: full state snapshot
  arrival_window: (earliest_time, latest_time)

HANDOFF_ACCEPT
  sender: node_id
  entity_id: id

HANDOFF_COMPLETE
  sender: node_id
  entity_id: id
  final_state: state at moment of crossing

BOUNDARY_EVENT
  sender: node_id
  event_type: explosion | sound | light_pulse | ...
  origin: position
  radius: effect radius
  parameters: event-specific data

LOAD_QUERY
  sender: node_id
  request: what is your current load?

LOAD_RESPONSE
  sender: node_id
  current_entities: count
  tick_duration_ratio: actual/target
  client_count: number of connected players
```

## 4.5 Node Splitting

When a node's domain becomes too loaded:

```
FUNCTION split_domain(node):

  -- Find the split plane (divide at the midpoint of the longest axis)
  split_axis = LONGEST axis of node.domain
  split_position = MIDPOINT of node.domain along split_axis

  -- Create two sub-domains
  domain_A = node.domain.left_half(split_axis, split_position)
  domain_B = node.domain.right_half(split_axis, split_position)

  -- Request a new node from orchestration
  new_node = orchestration.spawn_node(domain_B)
  WAIT for new_node to reach [STARTING] state

  -- Migrate entities in domain_B to new node
  entities_to_migrate = FILTER node.entities WHERE entity.position in domain_B
  FOR EACH entity E in entities_to_migrate:
    INITIATE handoff of E to new_node

  -- Update domain map
  orchestration.update_domain_map(node.id, domain_A)
  orchestration.update_domain_map(new_node.id, domain_B)

  -- Node now only handles domain_A
  node.domain = domain_A
```

## 4.6 Node Merging

When two adjacent nodes are both under-utilized:

```
FUNCTION merge_domains(node_A, node_B):
  -- node_A absorbs node_B's domain

  -- node_B begins draining
  node_B.status = DRAINING

  -- All clients connected to node_B transition to node_A
  FOR EACH client C connected to node_B:
    INITIATE client_transfer(C, node_A)

  -- All entities in node_B's domain transfer to node_A
  FOR EACH entity E in node_B.entities:
    INITIATE entity_transfer(E, node_A)

  WAIT for all transfers to complete

  -- Update domain map
  orchestration.update_domain_map(node_A.id, UNION of domain_A and domain_B)
  orchestration.remove_node(node_B.id)

  -- node_B shuts down
  node_B.status = STOPPED
```

---

# PART V: LOCAL ENGINE

## 5.1 Architecture Overview

The local engine is the client-side component. It runs on the player's machine. Its responsibilities are:

1. **Maintain a WebSocket connection** to the owning node
2. **Maintain a local world state** — a subset of the full world state relevant to this player
3. **Run local physics prediction** — predict how objects move between server updates
4. **Manage the asset cache** — store and retrieve geometry for all known object types
5. **Drive the LOD system** — determine what detail level to render each object at
6. **Render the visible world** — produce frames at high frequency
7. **Handle player input** — translate inputs into actions, send to node
8. **Manage node transitions** — handle handoffs transparently

## 5.2 The Client State Model

The client maintains a local world state that is a snapshot of what the server last told it, blended with local prediction. This state is organized as follows:

```
CLIENT WORLD STATE:
  player:
    position: (x, y, z)
    orientation: quaternion
    velocity: (vx, vy, vz)
    state: alive | dead | spectating | ...

  nearby_entities: map of entity_id -> entity_state
    FOR EACH entity E within visibility radius:
      E.position: last known + predicted delta
      E.orientation: last known + predicted rotation
      E.velocity: last known
      E.state: last known
      E.last_server_update: timestamp
      E.prediction_age: time since last server update

  nearby_objects: map of object_id -> object_state
    FOR EACH object O within visibility radius:
      O.position: authoritative (objects don't move without events)
      O.state: last known
      O.lod_tier: current assigned tier
      O.geometry: reference to asset in cache (or PENDING)

  terrain:
    loaded_chunks: map of chunk_id -> chunk_mesh
    pending_chunks: list of chunk_ids being streamed

  asset_cache:
    object_type_id -> geometry (indexed by LOD tier)
```

## 5.3 The Client Update Loop

```
FUNCTION client_update_loop():
  LOOP:
    dt = time_since_last_frame()

    -- Step 1: Process network messages from server
    messages = DRAIN network_receive_queue
    FOR EACH message M in messages:
      apply_server_message(M, client_world_state)

    -- Step 2: Process player input
    inputs = DRAIN input_queue
    player_action = RESOLVE inputs to action
    IF player_action exists:
      apply_locally(player_action, client_world_state)  -- optimistic
      ENQUEUE player_action for sending to server

    -- Step 3: Run local physics prediction
    FOR EACH entity E in nearby_entities:
      IF E.prediction_age > MAX_PREDICTION_AGE:
        -- Stop predicting, wait for server
        E.velocity = ZERO
      ELSE:
        E.position += E.velocity * dt
        E.prediction_age += dt

    -- Step 4: Apply gravity and simple collisions to locally predicted entities
    FOR EACH predicted entity E:
      IF E.position.y below terrain_height(E.position.x, E.position.z):
        E.position.y = terrain_height(E.position.x, E.position.z)
        E.velocity.y = 0

    -- Step 5: LOD update (don't run every frame — run every N frames)
    IF frame_number % LOD_UPDATE_INTERVAL == 0:
      lod_system.update(player.position, nearby_objects)

    -- Step 6: Render
    render_frame(client_world_state)

    -- Step 7: Send pending actions to server
    FLUSH action_queue to websocket

    WAIT until next_frame_time
```

## 5.4 Server Reconciliation (Prediction Correction)

When the server sends an authoritative state update that contradicts the local prediction:

```
FUNCTION reconcile(server_state, local_state):

  FOR EACH entity E in server_state:
    local_E = local_state.find(E.id)

    IF local_E not found:
      -- New entity appeared in range
      local_state.ADD(E)
      CONTINUE

    discrepancy = distance(E.position, local_E.position)

    IF discrepancy < SNAP_THRESHOLD:
      -- Small difference: smooth interpolation
      local_E.position = LERP(local_E.position, E.position, RECONCILE_SPEED)
    ELSE IF discrepancy < TELEPORT_THRESHOLD:
      -- Large difference: fast snap with brief visual effect
      local_E.position = LERP(local_E.position, E.position, FAST_RECONCILE_SPEED)
    ELSE:
      -- Extreme difference (possible lag spike): instant snap
      local_E.position = E.position

    local_E.velocity = E.velocity
    local_E.state = E.state
    local_E.prediction_age = 0

  FOR EACH entity E in local_state NOT in server_state:
    IF E is outside visibility radius:
      local_state.REMOVE(E)
```

---

# PART VI: LOD SYSTEM

## 6.1 Philosophy

The LOD system is one of the two most important systems in the engine for achieving scale (the other is the asset pipeline). Without LOD, rendering a world with millions of objects is impossible. With it, the per-frame cost grows with screen coverage, not with object count.

The LOD system's contract:
- Every visible object has exactly one LOD tier assigned per frame
- The assigned tier is determined solely by the object's distance from the camera
- Tier transitions are smooth — never abrupt visual pops
- The system integrates with the asset pipeline: if the required tier is not cached, it requests it
- The system can be configured globally (distance thresholds) and per object-type (some objects justify higher detail at distance)

## 6.2 LOD Tiers

Each object type defines up to 5 LOD tiers:

| Tier | Name | Typical Distance | Polygon Count |
|------|------|-----------------|---------------|
| 0 | Full Detail | 0 - 50 units | 1,000 - 100,000 |
| 1 | High | 50 - 200 units | 200 - 5,000 |
| 2 | Medium | 200 - 500 units | 50 - 500 |
| 3 | Low | 500 - 1,500 units | 10 - 100 |
| 4 | Impostor | 1,500 - 5,000 units | 2 (billboard) |
| ∞ | Invisible | beyond 5,000 units | 0 |

These thresholds are per-object-type defaults. High-importance objects (player characters, key structures) have their thresholds pushed further out. Low-importance objects (grass, pebbles) have thresholds pushed closer in.

## 6.3 LOD Assignment Algorithm

```
FUNCTION assign_lod_tier(object O, camera_position C):

  distance = distance(O.position, C)

  -- Get object type's distance thresholds
  thresholds = object_type_registry[O.type].lod_thresholds
  -- thresholds = [d0, d1, d2, d3, d4, d_invisible]

  -- Apply global LOD scale factor (quality setting)
  effective_distance = distance / GLOBAL_LOD_SCALE

  FOR tier = 0 to 4:
    IF effective_distance < thresholds[tier + 1]:
      target_tier = tier
      BREAK
  ELSE:
    target_tier = INVISIBLE

  RETURN target_tier
```

## 6.4 Smooth Tier Transitions

Avoid visual "popping" when an object crosses a tier boundary:

```
FUNCTION compute_lod_blend(object O, camera_position C):

  distance = distance(O.position, C)
  thresholds = object_type_registry[O.type].lod_thresholds

  -- Find the two tiers we're blending between
  current_tier = assign_lod_tier(O, C)
  next_tier = current_tier + 1

  -- Compute blend factor in the transition zone (last 20% of tier range)
  tier_start = thresholds[current_tier]
  tier_end = thresholds[current_tier + 1]
  transition_start = tier_start + (tier_end - tier_start) * 0.8

  IF distance > transition_start:
    blend_t = (distance - transition_start) / (tier_end - transition_start)
    RETURN render_blended(O, current_tier, next_tier, blend_t)
  ELSE:
    RETURN render_single_tier(O, current_tier)
```

## 6.5 LOD and Asset Requests

When the LOD system determines a tier that is not yet in the asset cache:

```
FUNCTION request_missing_lod_tier(object_type_id, tier):

  IF asset_cache.has(object_type_id, tier):
    RETURN CACHED

  IF asset_request_queue.already_queued(object_type_id, tier):
    RETURN PENDING

  -- Add to request queue with priority based on tier
  -- Lower tiers (higher detail) get higher priority if object is close
  priority = COMPUTE_PRIORITY(object_type_id, tier, object.distance)
  asset_request_queue.ADD(object_type_id, tier, priority)

  -- While waiting, render with the best available tier
  best_available_tier = asset_cache.best_available(object_type_id)
  RETURN render_single_tier(object, best_available_tier)
```

---

# PART VII: ASSET PIPELINE

## 7.1 The Core Insight

The asset pipeline is built around one insight: **serve geometry once, stream only state forever**.

Object geometry (shape, surface, animation skeleton) does not change unless the object's definition changes. The geometry of a "pine tree type A" is the same in the forest at position (100, 0, 200) as it is at position (50,000, 0, 99,999). Once a player's machine has that geometry, it can render any instance of "pine tree type A" anywhere in the world with zero additional bandwidth.

State (where this specific tree is, whether it's been cut down, what its health is) is tiny. A position is 12 bytes. A state enum is 1 byte. Geometry is kilobytes to megabytes.

This asymmetry is the foundation of the asset pipeline.

## 7.2 Asset Storage Architecture

Assets are stored in the asset store — a distributed object storage system separate from the world graph. The asset store is optimized for large binary objects (geometry, textures) with high read frequency and low write frequency.

Asset records:
```
ASSET RECORD:
  type_id: unique identifier for this object type
  version: increments when geometry changes
  geometry_tiers:
    tier_0: binary mesh data (highest detail)
    tier_1: binary mesh data
    tier_2: binary mesh data
    tier_3: binary mesh data
    tier_4: impostor image
  total_size: sum of all tiers in bytes
  created_at: timestamp
  last_updated: timestamp
```

## 7.3 Client-Side Asset Cache

The client maintains a persistent disk cache of downloaded assets. The cache is organized as:

```
ASSET CACHE STRUCTURE:
  index: map of (type_id, tier) -> cache_entry

  CACHE ENTRY:
    type_id: object type
    tier: LOD tier
    version: version of the stored asset
    file_path: where on disk the geometry is stored
    last_accessed: timestamp
    size_bytes: how much disk space this uses

  total_size_limit: configurable (default 10 GB)
  current_size: running total of all stored assets
```

**Cache eviction policy:**
```
WHEN current_size > total_size_limit:
  candidates = SORT index by last_accessed ascending
  WHILE current_size > target_size (80% of limit):
    entry = candidates.REMOVE_FIRST
    DELETE file at entry.file_path
    index.REMOVE(entry.type_id, entry.tier)
    current_size -= entry.size_bytes
```

**Cache validation:**
When the server sends a state update for an object of type T, it includes the current version of T's asset. If the client's cached version differs, the client queues an asset re-request.

```
ON server_state_update for object O:
  IF asset_cache.version(O.type_id) != O.asset_version:
    INVALIDATE asset_cache entry for O.type_id
    REQUEST new asset from server
```

## 7.4 Asset Streaming Protocol

Assets are served by the node over the same WebSocket connection as metadata, but in a separate lower-priority channel.

The protocol distinguishes two message classes:
- **Priority A (metadata)**: game state updates, entity positions, events — sent every tick, never delayed
- **Priority B (assets)**: geometry, textures — sent when bandwidth is available, can be delayed

The client's request queue for assets is ordered by priority:

```
ASSET REQUEST PRIORITY:
  priority = BASE_PRIORITY(tier) * PROXIMITY_FACTOR(distance)

  BASE_PRIORITY:
    tier 0 (full detail): 100
    tier 1 (high): 80
    tier 2 (medium): 50
    tier 3 (low): 30
    tier 4 (impostor): 10

  PROXIMITY_FACTOR:
    distance < 50: 3.0
    distance 50-200: 2.0
    distance 200-500: 1.5
    distance 500-1500: 1.0
    distance > 1500: 0.5
```

The server processes asset requests in priority order, sending asset chunks when there is bandwidth headroom after all priority-A metadata has been sent.

## 7.5 Asset Compression

Geometry is stored compressed. The compression algorithm is chosen for fast decompression (the client decompresses in real time as assets stream in), not maximum compression ratio.

- **Mesh geometry**: compressed using a mesh compression scheme that exploits the shared-vertex structure of triangle meshes. A typical 10,000-polygon mesh compresses to 10-30% of its uncompressed size.
- **Textures**: compressed using a GPU-native format (the specific format is an open ADR — ADR-005). GPU-native means the texture can be uploaded directly to GPU memory without CPU decompression.
- **Impostors**: stored as standard image format, resolution tuned so they look correct at their intended distance.

---

# PART VIII: NETWORK PROTOCOL

## 8.1 Design Principles

The network protocol is designed around five principles:

1. **Typed**: Every message has a type identifier. The receiver never has to guess what a message contains.
2. **Versioned**: Every message type has a version number. Old clients can reject new message types gracefully. New clients can reject old message types gracefully.
3. **Binary**: Game state is transmitted as compact binary, not text. This reduces bandwidth and parsing overhead significantly.
4. **Prioritized**: Metadata (game state) has higher priority than assets (geometry). The transport layer enforces this.
5. **Acknowledged for critical changes**: State changes that must not be lost (player actions with permanent consequences) require explicit acknowledgment from the server.

## 8.2 Message Framing

Every message over the WebSocket connection is framed as:

```
BINARY MESSAGE FRAME:
  [2 bytes] message_type: identifies the message schema
  [2 bytes] message_version: version of this message type's schema
  [4 bytes] sequence_number: monotonically increasing per connection
  [4 bytes] timestamp_ms: server time when message was sent
  [4 bytes] payload_length: size of the following payload in bytes
  [N bytes] payload: the message content, schema defined by message_type
```

## 8.3 Message Types

**Metadata Messages (Priority A):**

```
ENTITY_POSITION_UPDATE (type: 0x0001):
  entity_count: uint16
  FOR EACH entity:
    entity_id: uint32
    position_x: float32
    position_y: float32
    position_z: float32
    orientation_w: float16  (quaternion, compressed)
    orientation_x: float16
    orientation_y: float16
    orientation_z: float16
    velocity_x: float16
    velocity_y: float16
    velocity_z: float16

OBJECT_STATE_CHANGE (type: 0x0002):
  object_id: uint32
  change_type: uint8  (enum: position | property | destruction | creation)
  payload: variable, defined by change_type

WORLD_EVENT (type: 0x0003):
  event_type: uint16
  origin_x: float32
  origin_y: float32
  origin_z: float32
  radius: float32
  payload: variable, defined by event_type

TICK_SYNC (type: 0x0004):
  server_tick: uint64
  server_time_ms: uint64
  -- Clients use this to synchronize their local clock with server time

PLAYER_JOINED (type: 0x0005):
  entity_id: uint32
  initial_state: full entity state

PLAYER_LEFT (type: 0x0006):
  entity_id: uint32
  reason: uint8  (disconnect | handoff | death)
```

**Control Messages (Priority A):**

```
HANDSHAKE (type: 0x0100):
  -- First message sent by client to server
  client_version: uint32
  player_id: uint64
  auth_token: 32 bytes (cryptographic authentication)

HANDSHAKE_RESPONSE (type: 0x0101):
  status: uint8  (accepted | rejected | version_mismatch)
  IF accepted:
    assigned_entity_id: uint32
    initial_position: 3x float32
    server_version: uint32
    world_seed: uint64

NODE_TRANSFER (type: 0x0102):
  new_node_address: variable length string
  transfer_token: 32 bytes

ACTION_ACKNOWLEDGMENT (type: 0x0103):
  sequence_number: uint32  (matches the sequence number of the acknowledged action)
  result: uint8  (success | rejected | partial)
  payload: variable, result details

ERROR (type: 0x01FF):
  error_code: uint16
  message_length: uint16
  message: utf8 string
```

**Client-to-Server Messages:**

```
PLAYER_ACTION (type: 0x0200):
  action_type: uint16  (move | interact | build | destroy | communicate | ...)
  sequence_number: uint32  (client generates, used for acknowledgment)
  requires_ack: uint8  (1 = server must acknowledge, 0 = fire and forget)
  payload: variable, defined by action_type

ASSET_REQUEST (type: 0x0201):
  request_count: uint8
  FOR EACH request:
    object_type_id: uint32
    lod_tier: uint8

CHAT_MESSAGE (type: 0x0202):
  channel: uint8  (local | global | group | ...)
  message_length: uint16
  message: utf8 string
```

**Asset Messages (Priority B):**

```
ASSET_CHUNK (type: 0x0300):
  object_type_id: uint32
  lod_tier: uint8
  asset_version: uint32
  chunk_index: uint32
  total_chunks: uint32
  chunk_data: variable bytes

ASSET_COMPLETE (type: 0x0301):
  object_type_id: uint32
  lod_tier: uint8
  asset_version: uint32
  total_size_bytes: uint32
  checksum: 4 bytes
```

## 8.4 Delta Compression

Entity positions are transmitted using delta compression when possible:

```
FUNCTION compress_position_update(current_positions, previous_positions):

  message = new ENTITY_POSITION_UPDATE

  FOR EACH entity E in current_positions:
    prev = previous_positions[E.id]

    IF prev not found OR distance(E.position, prev.position) > DELTA_THRESHOLD:
      -- Send full position
      message.ADD(E.id, FULL_POSITION, E.position, E.orientation, E.velocity)
    ELSE:
      -- Send delta (much smaller)
      delta_position = E.position - prev.position
      message.ADD(E.id, DELTA_POSITION, delta_position)
      -- Only include orientation/velocity if they changed significantly
      IF angle_diff(E.orientation, prev.orientation) > ORIENTATION_THRESHOLD:
        message.APPEND_ORIENTATION(E.id, E.orientation)
      IF speed_diff(E.velocity, prev.velocity) > VELOCITY_THRESHOLD:
        message.APPEND_VELOCITY(E.id, E.velocity)

  RETURN message
```

## 8.5 Bandwidth Budget

Per connected player, the bandwidth budget is:

| Channel | Bytes per tick | At 50 ticks/sec | Monthly at 100% uptime |
|---------|---------------|-----------------|------------------------|
| Metadata (server→client) | 200-2,000 | 10-100 KB/s | 26-260 GB |
| Player actions (client→server) | 20-200 | 1-10 KB/s | 2.6-26 GB |
| Asset streaming | 0-50,000 | 0-2.5 MB/s (burst) | Depends on world traversal |

Asset streaming is the wildcard. A player exploring new territory may download many megabytes per minute of new geometry. A player in a familiar region downloads almost nothing.

---

# PART IX: WORLD GRAPH AND OBJECT DATABASE

## 9.1 What The World Graph Must Do

The world graph is the persistent memory of the universe. It must:

1. **Store every object** with its full state, forever, durably
2. **Answer spatial queries** efficiently: "what is within R of position P"
3. **Store relationships** between objects: ownership, construction history, proximity, semantic similarity
4. **Support concurrent writes** from thousands of nodes simultaneously without corruption
5. **Scale horizontally** as the world grows: adding more data should not slow down existing queries
6. **Expose event streams** so nodes can subscribe to changes in their domain
7. **Maintain the ticker log** — an append-only history of every state change

## 9.2 Object Record Schema

Every object in the world graph has this record:

```
OBJECT RECORD:

  IDENTITY:
    id: uint64 — globally unique, never reused
    type_id: uint32 — references object type definition
    created_at: timestamp
    created_by: player_id | node_id | world_seed
    version: uint64 — increments on every state change

  SPATIAL:
    position: (float64, float64, float64) — high precision for large world
    orientation: quaternion (float32 x4)
    bounding_box: (min_x, min_y, min_z, max_x, max_y, max_z)
    domain_id: current owning node's domain identifier
    sector_coords: (int32, int32, int32) — which sector this is in
    chunk_coords: (int32, int32, int32) — which chunk within sector

  STATE:
    properties: map of property_name -> typed_value
    -- Properties are defined by the object type, not hardcoded in the schema
    -- Example: health: 100, fuel: 0.5, open: false, owner_id: 12345
    state_enum: uint8 — current life-cycle state (active | damaged | destroyed | ...)

  GRAPH:
    neighbors: list of (object_id, relationship_type, weight)
    -- Spatial neighbors updated by indexer when position changes
    -- Relational neighbors (ownership, construction, etc.) updated by simulation
    cluster_id: which semantic cluster this object belongs to
    semantic_vector: 5 float32 values (see knowledge-graph concepts in Appendix B)

  ASSETS:
    asset_version: uint32 — current version of geometry
    -- (actual geometry stored in asset store, not in world graph)

  BEHAVIOR:
    embedded_instructions: variable-length binary
    -- Instructions for AI agents that interact with this object
    -- Format defined by the agent system (Part XII)

  HISTORY:
    ticker_log_pointer: offset into the ticker log for this object's first entry
    last_event_at: timestamp
    state_change_count: uint64
```

## 9.3 Spatial Indexing

The world graph maintains a spatial index that allows fast proximity queries. The index is implemented as a hierarchical spatial structure (the exact choice is ADR-002 in Appendix C — candidates are R-tree, octree, spatial hash grid, or Hilbert curve partitioning).

Requirements for the spatial index:
- **Insert**: O(log N) amortized
- **Delete**: O(log N) amortized
- **Point query** (what object is at exactly this position): O(log N)
- **Radius query** (what objects are within R of P): O(log N + K) where K is result count
- **Box query** (what objects are within this bounding box): O(log N + K)
- **Nearest-K query** (what are the K nearest objects to P): O(K log N)

## 9.4 The Ticker Log

The ticker log is an append-only sequence of world events. It is the foundational audit and replay mechanism.

```
TICKER ENTRY:
  sequence: uint64 — global monotonic sequence number
  timestamp: uint64 — server time in microseconds since epoch
  object_id: uint64 — which object changed
  event_type: uint16 — what kind of change
  source: player_id | node_id | system — who caused it
  previous_state_snapshot: compressed snapshot of relevant previous state
  new_state_snapshot: compressed snapshot of new state
  session_id: uint64 — which server session this occurred in
```

The ticker log is partitioned by time and by sector. Queries for "what happened in sector S during time range T1 to T2" only scan the relevant partition.

**Ticker log operations:**
```
APPEND entry:  O(1) — always appended to end of current partition
QUERY by object_id in time range: O(log P + E) where P is partitions scanned, E is entries found
QUERY by sector in time range: O(log P + E)
REPLAY from timestamp T: O(log P + total entries after T) — full sequential scan
```

## 9.5 Relationship Types

Objects in the world graph are connected by typed relationships:

```
RELATIONSHIP TYPES:

  SPATIAL:
    near: bidirectional — these two objects are within X units
    contains: A contains B (B is inside A, e.g., a chest contains items)
    adjacent: A and B share a surface (for structural objects)

  OWNERSHIP:
    owned_by: A is owned by player P
    built_by: A was constructed by player P
    member_of: A belongs to group G

  CONSTRUCTION:
    made_from: A was constructed from B (B may be consumed)
    component_of: A is a component of B
    powers: A provides power to B
    connected_to: A and B are connected (pipes, wires, etc.)

  CAUSALITY:
    caused: event E caused change to object A
    triggered_by: A's current state was triggered by event E

  SEMANTIC:
    similar_to: A and B are semantically related (same type family, similar function)
    contrasts_with: A and B are opposites in some dimension
```

## 9.6 World Graph Partitioning

The world graph is partitioned by sector. Each sector's data is stored on a dedicated shard. This has important properties:
- All objects in the same sector are co-located in the same shard — spatial queries within a sector don't cross shard boundaries
- Cross-sector queries (rare) require a scatter-gather operation across relevant shards
- A node's simulation data (all objects in its domain) is typically within 1-4 shards
- Shards can be added as the world grows without migrating existing data

**Shard assignment:**
```
FUNCTION shard_for_sector(sector_x, sector_y, sector_z):
  -- Sectors map to shards using a consistent hash
  -- This ensures adjacent sectors tend to be on the same shard
  key = HILBERT_CURVE_INDEX(sector_x, sector_y, sector_z)
  shard_id = key / SECTORS_PER_SHARD
  RETURN shard_registry[shard_id]
```

---

# PART X: SIMULATION SYSTEM

## 10.1 What The Simulation Computes

Each tick, the simulation system computes the new state of the world within a node's domain. It does this in phases:

1. **Force integration**: Apply all forces (gravity, player movement intent, explosions, springs) to physics bodies
2. **Collision detection**: Find all pairs of objects whose bounding volumes overlap
3. **Collision resolution**: Compute impulses to separate colliding objects and conserve momentum
4. **Entity AI**: Run one AI step for each active entity
5. **World interactions**: Process all queued player actions
6. **State transitions**: Advance any timed state machines (growing crops, charging devices, etc.)

## 10.2 Physics Model

The physics model covers three categories of objects:

**Category A: Dynamic bodies** — things that move under forces
- Have mass, velocity, angular velocity, center of mass
- Affected by gravity, player forces, explosions
- Examples: players, animals, thrown objects, vehicles

**Category B: Static bodies** — things that occupy space but don't move
- Have position and collision shape
- Cannot be moved by physics — only by explicit events
- Examples: terrain, placed structures, large rocks

**Category C: Kinematic bodies** — things that move but ignore physics forces
- Have position and velocity but no mass
- Moved by scripted motion or AI behavior, not forces
- Examples: doors, elevators, moving platforms, vehicles on rails

**Force Integration (Pseudocode):**
```
FUNCTION integrate_dynamic_body(body B, dt):

  -- Accumulate forces
  total_force = GRAVITY * B.mass
  total_force += B.applied_force  -- from player input, explosions, etc.

  -- Compute acceleration
  acceleration = total_force / B.mass

  -- Integrate velocity (semi-implicit Euler — stable for game physics)
  B.velocity += acceleration * dt

  -- Apply damping (prevents infinite acceleration in weird cases)
  B.velocity *= (1.0 - DAMPING_COEFFICIENT * dt)

  -- Integrate position
  B.position += B.velocity * dt

  -- Angular physics
  total_torque = B.applied_torque
  angular_acceleration = total_torque / B.moment_of_inertia
  B.angular_velocity += angular_acceleration * dt
  B.angular_velocity *= (1.0 - ANGULAR_DAMPING * dt)
  B.orientation = INTEGRATE_QUATERNION(B.orientation, B.angular_velocity, dt)

  -- Clear applied forces (they must be re-applied each tick)
  B.applied_force = ZERO
  B.applied_torque = ZERO
```

## 10.3 Collision Detection

Collision detection uses a two-phase approach:

**Phase 1: Broad phase** — quickly eliminate pairs of objects that are definitely not colliding
```
FUNCTION broad_phase(all_bodies):
  -- Sort bodies along one axis (sweep and prune)
  -- Only pairs whose projections on that axis overlap are candidates
  candidates = SORT_AND_SWEEP(all_bodies, axis=X)
  RETURN candidates
```

**Phase 2: Narrow phase** — exact collision test for candidate pairs
```
FUNCTION narrow_phase(body_A, body_B):
  -- Use GJK algorithm for convex shapes
  -- Use SAT (Separating Axis Theorem) for simple boxes and spheres
  -- Use mesh-vs-mesh only as last resort (expensive)

  IF both A and B are SPHERE:
    RETURN sphere_sphere_test(A, B)
  ELSE IF both are BOX:
    RETURN box_box_test(A, B)  -- SAT
  ELSE IF one is SPHERE, other is BOX:
    RETURN sphere_box_test(A, B)
  ELSE:
    RETURN gjk_test(A.convex_hull, B.convex_hull)
```

## 10.4 Entity AI Behavior System

Each entity type has a behavior definition — a tree of conditions and actions:

```
BEHAVIOR DEFINITION STRUCTURE:
  name: string
  priority: uint8 — higher priority behaviors interrupt lower ones
  conditions: list of condition expressions
  actions: list of action descriptors
  sub_behaviors: list of nested behavior definitions

CONDITION TYPES:
  in_range_of(target_type, radius): is there a target of given type within radius?
  has_property(property, comparator, value): e.g., health < 20
  time_elapsed(duration): has this behavior been active for this long?
  has_path_to(target): is there a navigable path to the target?

ACTION TYPES:
  move_toward(target, speed)
  move_away_from(target, speed)
  idle(duration)
  interact_with(target, interaction_type)
  set_property(property, value)
  emit_event(event_type, payload)
  transition_behavior(next_behavior_name)
```

**Behavior evaluation per tick (Pseudocode):**
```
FUNCTION evaluate_entity_behavior(entity E, world_state):

  current_behavior = E.active_behavior

  -- Check if a higher-priority behavior should interrupt
  FOR behavior B in E.behavior_definitions SORTED BY priority DESC:
    IF B.priority > current_behavior.priority:
      IF ALL conditions in B.conditions are TRUE (evaluated against world_state):
        -- Interrupt current behavior
        current_behavior.on_exit(E)
        E.active_behavior = B
        B.on_enter(E)
        current_behavior = B
        BREAK

  -- Execute the current behavior's actions
  FOR action A in current_behavior.actions:
    A.execute(E, world_state)

  -- Check if current behavior is complete
  IF current_behavior.is_complete(E):
    next = current_behavior.transitions.evaluate(E, world_state)
    current_behavior.on_exit(E)
    E.active_behavior = next
    next.on_enter(E)
```

---

# PART XI: PLAYER SYSTEM

## 11.1 The Player As An Entity

A player is an entity — it has all the properties of an entity plus additional player-specific properties:

```
PLAYER ENTITY:
  -- All entity fields plus:
  player_id: uint64 — persistent across sessions
  display_name: string
  account_level: uint32

  AVATAR:
    avatar_type_id: uint32 — what the player looks like
    equipment: list of equipped item_ids
    cosmetics: list of active cosmetic_ids

  INVENTORY:
    items: list of item records
    maximum_weight: float32
    current_weight: float32

  SKILLS:
    skill_levels: map of skill_id -> level
    active_abilities: list of ability_ids

  SESSION:
    connected_node_id: current node
    last_position: (x, y, z) — saved on disconnect
    last_orientation: quaternion
    session_start: timestamp
    accumulated_play_time: duration

  SOCIAL:
    guild_id: optional
    friends: list of player_ids
    blocked: list of player_ids
```

## 11.2 Player Input Processing

Player input arrives at the server as ACTION messages. The server processes them in the simulation tick:

```
ACTION TYPES:

MOVE_INTENT:
  direction: unit vector (3 floats)
  -- Player is pressing a movement key in this direction
  -- Server computes actual velocity based on direction, speed, terrain
  -- Does NOT directly set position (server is authoritative on physics)

LOOK_DIRECTION:
  orientation: quaternion
  -- Where the player is looking
  -- Immediately applied for client prediction; server validates reasonable changes

INTERACT:
  target_object_id: uint64
  interaction_type: uint16  (use | pick_up | inspect | build_on | ...)
  parameters: variable

BUILD:
  object_type_id: uint32
  position: (x, y, z)
  orientation: quaternion
  -- Server validates: player has materials, position is valid, not overlapping

DESTROY:
  target_object_id: uint64
  -- Server validates: player has permission, object is destructible

EQUIP:
  item_id: uint64
  slot: uint8

DROP:
  item_id: uint64
  position: (x, y, z)

COMMUNICATE:
  channel: uint8
  message: utf8 string
```

## 11.3 The Visibility Radius

Each player has a visibility radius — the distance at which they receive metadata about objects and entities. Beyond this radius, the server sends no updates about a given entity.

The visibility radius is not fixed — it adapts based on:
- Server load (if the node is overloaded, visibility radii contract)
- Player speed (a fast-moving player needs a larger radius to avoid pop-in)
- Object type (important objects like other players have a larger effective radius)

```
FUNCTION compute_visibility_radius(player P, node_load L):
  base_radius = PLAYER_BASE_VISIBILITY_RADIUS  -- e.g., 500 units

  -- Reduce radius under high load
  load_factor = 1.0 - (L.tick_duration_ratio - 1.0) * LOAD_SCALE_FACTOR
  load_factor = CLAMP(load_factor, MIN_VISIBILITY_FACTOR, 1.0)

  -- Increase radius for fast-moving players
  speed = LENGTH(P.velocity)
  speed_factor = 1.0 + (speed / MAX_EXPECTED_SPEED) * SPEED_SCALE_FACTOR

  effective_radius = base_radius * load_factor * speed_factor
  RETURN effective_radius
```

---

# PART XII: AGENT SYSTEM

## 12.1 What An Agent Is

An agent is an entity with embedded instructions. Unlike player-controlled entities (which receive movement from a human via WebSocket), agents receive their movement and actions from their behavior definition evaluated on the server.

Agents can:
- Perceive the world within a radius (same mechanism as the visibility system)
- Move through the world (same physics as player entities)
- Interact with objects (same interaction system)
- Communicate (same channels as players)
- Create and destroy objects (with appropriate permissions)
- Have goals that persist across ticks

Agents cannot:
- Access any information beyond their perception radius directly
- Write to the world graph except through the normal interaction system
- Bypass physics

## 12.2 Agent Perception

Each tick, an agent's perception module queries the world state:

```
AGENT PERCEPTION (per tick):

  -- Spatial query within perception radius
  visible_objects = QUERY world_state for objects within agent.perception_radius of agent.position

  -- Filter by line-of-sight (optional, expensive — not all agent types use it)
  IF agent.uses_line_of_sight:
    visible_objects = FILTER visible_objects WHERE
      ray_cast(agent.position, object.position) is not blocked

  -- Organize perception into semantic categories
  perception = {
    nearby_players: FILTER visible_objects WHERE type is PLAYER
    nearby_agents: FILTER visible_objects WHERE type is AGENT
    nearby_items: FILTER visible_objects WHERE type is ITEM
    nearby_terrain: terrain data in radius
    nearby_events: events that occurred in last N ticks within radius
    self: agent's own state
  }

  RETURN perception
```

## 12.3 Agent Decision Making

Agents use the behavior tree system (defined in Part X, Section 10.4) as their base decision system. For advanced agents, the behavior tree is enhanced with a goal-oriented action planning layer:

```
GOAL-ORIENTED ACTION PLANNING (GOAP):

  AGENT has:
    current_world_state: relevant subset of world state as key-value pairs
    goal_state: desired world state as key-value pairs
    available_actions: list of action definitions

  EACH ACTION DEFINITION has:
    preconditions: required current_world_state values
    effects: changes this action makes to current_world_state
    cost: computational estimate of how expensive this action is

  PLANNING:
    FIND the sequence of actions that transforms current_world_state into goal_state
    with minimum total cost, respecting preconditions

    This is a graph search problem:
    nodes = world states
    edges = actions
    START = current_world_state
    GOAL = any state where goal_state is satisfied
    ALGORITHM = A* with cost as path metric

  EXECUTION:
    Execute the planned action sequence one action per tick
    After each action, re-evaluate: is the goal still valid? Is there a better plan?
    If world state changed unexpectedly, replan
```

---

# PART XIII: WORLD ECONOMY AND BUILDING SYSTEM

## 13.1 Resources and Items

The world contains resources that players can gather. Resources are objects with specific types:

```
RESOURCE TYPES:
  raw materials: stone, wood, metal ore, fiber, crystal, energy cells, ...
  processed materials: planks, ingots, fabric, refined crystals, ...
  components: gears, circuits, pipes, containers, ...
  tools: axe, pick, wrench, scanner, ...
  consumables: food, fuel, medicine, ...
  information: data crystals, schematics, maps, ...
```

Resources are gathered through interaction. The gathering system:

```
FUNCTION gather_resource(player P, resource_object R):

  IF distance(P.position, R.position) > INTERACTION_RANGE:
    RETURN error(TOO_FAR)

  IF NOT has_required_tool(P, R.type):
    RETURN error(MISSING_TOOL)

  IF P.inventory.current_weight + R.item_weight > P.inventory.maximum_weight:
    RETURN error(INVENTORY_FULL)

  -- Compute yield based on player skill and tool quality
  base_yield = R.properties.quantity
  skill_bonus = P.skills[R.type.required_skill].level * SKILL_YIELD_FACTOR
  tool_bonus = get_equipped_tool(P, R.type).quality * TOOL_YIELD_FACTOR
  actual_yield = base_yield * (1.0 + skill_bonus + tool_bonus)

  -- Transfer resource to player inventory
  item = CREATE item(type=R.type, quantity=actual_yield)
  P.inventory.ADD(item)

  -- Update resource object
  R.properties.quantity -= base_yield  -- yield removes from base, not bonus
  IF R.properties.quantity <= 0:
    DESTROY R (mark as destroyed in world graph, remove from octree)
  ELSE:
    WRITE R to world graph

  -- Record in ticker log
  TICK: player P gathered actual_yield of R.type from object R.id

  -- Award skill XP
  P.skills[R.type.required_skill].xp += XP_PER_GATHER
```

## 13.2 The Building System

Players can place objects in the world. The building system:

```
FUNCTION place_object(player P, object_type T, position pos, orientation orient):

  -- Validate placement
  IF distance(P.position, pos) > BUILD_RANGE:
    RETURN error(TOO_FAR)

  -- Check for overlap with existing objects
  candidates = QUERY octree for objects within T.bounding_box at pos
  FOR EACH candidate C in candidates:
    IF bounding_boxes_overlap(T.bounding_box at pos, C.bounding_box):
      IF NOT C.type.allows_overlap:
        RETURN error(PLACEMENT_BLOCKED, C.id)

  -- Check player has required materials
  recipe = crafting_registry[T.id]
  FOR EACH ingredient in recipe.ingredients:
    IF NOT P.inventory.has(ingredient.type, ingredient.quantity):
      RETURN error(MISSING_MATERIAL, ingredient.type)

  -- Consume materials
  FOR EACH ingredient in recipe.ingredients:
    P.inventory.REMOVE(ingredient.type, ingredient.quantity)

  -- Create the object
  new_object = CREATE object(
    type_id: T.id,
    position: pos,
    orientation: orient,
    created_by: P.player_id,
    properties: T.default_properties
  )

  -- Insert into world graph and octree
  world_graph.INSERT(new_object)
  node_octree.INSERT(new_object)

  -- Establish relationships
  world_graph.ADD_RELATIONSHIP(new_object.id, P.player_id, BUILT_BY)

  -- Broadcast to nearby clients
  BROADCAST object_created event to nearby players

  -- Ticker log
  TICK: player P placed T.name at position pos

  RETURN success(new_object.id)
```

## 13.3 Functional Machines

The most complex objects in the world are functional machines — objects that process inputs to produce outputs. Machines are implemented as state machines with a simulation step:

```
MACHINE DEFINITION:
  inputs: list of (slot_name, accepted_types, max_quantity)
  outputs: list of (slot_name, produced_types)
  process: a sequence of processing steps
  energy_per_tick: energy consumed when active
  ticks_per_cycle: how many ticks one processing cycle takes

MACHINE STATE:
  active: bool
  input_slots: map of slot_name -> current_items
  output_slots: map of slot_name -> current_items
  current_tick: progress through current cycle
  energy_stored: current energy level

MACHINE SIMULATION (per tick, in simulation phase):
  IF machine.active AND machine.energy_stored >= machine.energy_per_tick:
    machine.energy_stored -= machine.energy_per_tick
    machine.current_tick += 1

    IF machine.current_tick >= machine.ticks_per_cycle:
      -- Process is complete
      IF inputs_are_available(machine) AND outputs_have_space(machine):
        consume_inputs(machine)
        produce_outputs(machine)
        machine.current_tick = 0
      ELSE:
        machine.active = false  -- stall: no inputs or output full
  ELSE IF machine.energy_stored < machine.energy_per_tick:
    machine.active = false  -- stall: no energy
```

## 13.4 In-World Computers

A special category of machines are computers — functional devices that run programs within the game world. A computer is a machine whose "processing" is computational: it takes data inputs and produces data outputs according to a program.

The computer system is intentionally the most complex object type in the engine because it is the mechanism for emergent complexity — players building programmable systems that interact with the world.

```
COMPUTER OBJECT:
  processing_power: float — how many instructions per tick it can execute
  memory: amount of data it can hold
  program_slot: the current program loaded (null if none)
  data_inputs: connected data sources (other computers, sensors, world queries)
  data_outputs: connected data destinations (actuators, displays, other computers)

PROGRAM RECORD (stored in world graph):
  source: a sequence of operations in the world's native computation language
  -- NOTE: the computation language is specified in a separate ADR (ADR-010)
  -- It is intentionally constrained: no network access, no file system,
  -- only the APIs the engine provides

COMPUTER SIMULATION (per tick):
  IF computer.program_slot is not null AND computer.energy_stored > 0:
    execute program for computer.processing_power instructions
    -- Program can:
    --   read from data_inputs
    --   write to data_outputs
    --   query nearby objects via engine API
    --   trigger actions on connected machines
    --   store data in computer's memory
```

---

# PART XIV: PLATFORM AND ORCHESTRATION

## 14.1 The Deployment Model

The game engine runs in a cluster of containers. Each container is one of:
- **Node container**: runs one node process, owns one domain
- **World graph shard**: runs one database shard
- **Asset store shard**: runs one asset store partition
- **Orchestration controller**: manages the node cluster
- **Edge gateway**: accepts player WebSocket connections, routes to correct node
- **Ticker log consumer**: reads the ticker log and drives analytics, replay, etc.

## 14.2 The Orchestration System

The orchestration system is the cluster brain. It:
- Maintains the domain map (which node owns which domain)
- Monitors node health and load
- Spawns new nodes when domains are overloaded
- Terminates idle nodes
- Assigns nodes to containers
- Manages the scaling of world graph shards

```
ORCHESTRATION TICK (runs every 10 seconds):

  FOR EACH active node N:
    IF N.tick_duration_ratio > SPLIT_THRESHOLD and N has been overloaded for OVERLOAD_GRACE_PERIOD:
      REQUEST_SPLIT(N)

    IF N.tick_duration_ratio < MERGE_THRESHOLD and N has been underloaded for IDLE_GRACE_PERIOD:
      -- Find adjacent underloaded node
      adjacent = FIND adjacent node with lowest load
      IF adjacent.tick_duration_ratio < MERGE_THRESHOLD:
        REQUEST_MERGE(N, adjacent)

    IF N.health_check FAILS for 3 consecutive checks:
      MARK N as failed
      ASSIGN N's domain to adjacent nodes temporarily
      SPAWN replacement node for N's domain

  -- Scale world graph shards if needed
  FOR EACH shard S:
    IF S.query_latency > TARGET_LATENCY:
      IF S.sector_count > MIN_SECTORS_FOR_SPLIT:
        REQUEST_SHARD_SPLIT(S)
```

## 14.3 The Edge Gateway

Players do not connect directly to nodes. They connect to edge gateways — load balancers that sit in front of the node cluster.

```
PLAYER CONNECTION FLOW:

  1. Player's client opens WebSocket to nearest edge gateway
     (DNS routing or explicit server selection)

  2. Edge gateway receives HANDSHAKE message
     EXTRACT player_id and auth_token
     VALIDATE auth_token against auth service
     IF invalid: CLOSE connection

  3. Edge gateway queries domain map:
     player_last_position = QUERY auth service for player P's last known position
     owning_node = find_node_for_position(player_last_position)

  4. Edge gateway proxies the connection to owning_node
     OR redirects client to connect directly to owning_node
     (decision based on network topology — ADR-007)

  5. Node handles the connection directly from this point
```

## 14.4 Geographic Distribution

For a global player base, nodes must be geographically distributed. A player in Europe should connect to a server in Europe. A player in Asia should connect to a server in Asia. But the world is one world — objects built by a European player and objects built by an Asian player coexist.

This creates the cross-region problem: how does a European player interact with an object near an Asian player?

```
CROSS-REGION ARCHITECTURE:

  The world is geographically partitioned into regions.
  Each geographic region has its own cluster of nodes and world graph shards.
  A player connects to the cluster in their geographic region.

  Objects do not move between geographic regions on their own —
  they live in the geographic region where they were created.

  When a player's avatar moves across geographic boundaries
  (this is intentional world design — geographic region boundaries are far apart,
  e.g., one per continent):
    The player's connection migrates to the new geographic region's cluster.
    This handoff uses the same mechanism as node handoff, but cross-cluster.
    Cross-cluster handoffs have higher latency than intra-cluster handoffs.
    The world design should minimize the frequency of cross-cluster handoffs.

  The ticker log is replicated across geographic regions with a delay.
  This means geographic regions have eventually consistent views of each other.
  Eventual consistency is acceptable for distant world state but not for
  close-range interaction (which always routes to the local cluster).
```

---

# PART XV: MODULARITY SPECIFICATION

## 15.1 The Layering Rule

The engine is organized in layers. Each layer can only depend on layers below it. No layer can depend on a layer above it.

```
LAYER 6: World Logic (farming, crafting, building, quests)
  └── depends on Layer 5

LAYER 5: Game Entities (player, agent, machine, item)
  └── depends on Layer 4

LAYER 4: World Systems (LOD, asset streaming, node resolver, event bus)
  └── depends on Layer 3

LAYER 3: Engine Primitives (mesh, shader, physics body, socket connection, world graph client)
  └── depends on Layer 2

LAYER 2: Platform Abstractions (WebGL wrapper, WebSocket wrapper, file system, database client)
  └── depends on Layer 1

LAYER 1: Platform (WebGL, WebSockets, operating system, hardware)
  └── depends on Layer 0

LAYER 0: Hardware / Runtime (GPU, CPU, network, disk)
  -- no dependencies within the engine
```

**The rule for all design decisions**: If a piece of behavior would require a higher layer to depend on a lower layer in a reverse direction, the design is wrong and must be restructured.

## 15.2 Subsystem Contracts

Every subsystem publishes a contract — a description of what it accepts and what it produces. No subsystem knows the internals of any other. They communicate only through contracts.

The contracts are maintained in `programs/game_engine/shared/contracts/`. Currently defined contracts:

```
world-state-contract.md
  PROVIDES: read access to world state (objects, entities, terrain)
  ACCEPTS: state change requests (with source identity)
  DOES NOT: execute changes directly (changes go through simulation)

node-registry-contract.md
  PROVIDES: domain-to-node mapping, node health status
  ACCEPTS: node registration, node load reports, domain map update requests
  DOES NOT: manage node processes (that is orchestration's job)

asset-store-contract.md
  PROVIDES: asset retrieval by (type_id, lod_tier, version)
  ACCEPTS: asset upload (new or updated geometry)
  DOES NOT: know which players need which assets

lod-system-contract.md
  PROVIDES: LOD tier assignment for a given (object_type, distance)
  ACCEPTS: configuration updates (distance thresholds, quality scale)
  DOES NOT: trigger asset requests (it signals the need; the asset pipeline acts)

simulation-contract.md
  PROVIDES: tick execution, physics integration, collision results
  ACCEPTS: force application, entity behavior trees, world interaction requests
  DOES NOT: write to world graph directly (it produces changes; the node writes them)

player-session-contract.md
  PROVIDES: authenticated player identity, current position, inventory state
  ACCEPTS: player action messages (validated and typed)
  DOES NOT: handle network transport (that is the WebSocket layer's job)
```

## 15.3 Extension Points

The engine is designed to be extended without modification. Extension points are interfaces at which new implementations can be substituted:

```
EXTENSION POINTS:

world-generator:
  INTERFACE: given sector coordinates, produce a sector description
  DEFAULT: procedural noise-based generator
  EXTENSIBLE BY: any generator that implements the interface

entity-behavior:
  INTERFACE: given entity + world state, produce a list of actions
  DEFAULT: behavior tree evaluator
  EXTENSIBLE BY: GOAP planner, neural network, scripted sequence, ...

object-type-registry:
  INTERFACE: given type_id, return object type definition
  DEFAULT: built-in registry loaded from world graph
  EXTENSIBLE BY: plugin-supplied type definitions

physics-integrator:
  INTERFACE: given body list + dt, produce new body states
  DEFAULT: semi-implicit Euler with BVH collision
  EXTENSIBLE BY: any integrator (Verlet, RK4, constraint-based, ...)

asset-format:
  INTERFACE: given a binary blob, produce renderable geometry
  DEFAULT: engine's native compressed mesh format
  EXTENSIBLE BY: any format converter

network-transport:
  INTERFACE: bidirectional typed message stream
  DEFAULT: WebSocket with binary framing
  EXTENSIBLE BY: WebRTC, QUIC, or any transport that provides the interface
```

---

# PART XVI: PERFORMANCE CONTRACTS

## 16.1 The Performance Budget

Performance contracts are non-negotiable requirements. A design that cannot meet these contracts is an invalid design — it must be changed, not the contracts.

**Client Performance:**

| Metric | Requirement | Priority |
|--------|-------------|----------|
| Frame rate | 60 FPS minimum on target hardware | P0 |
| Frame rate | 30 FPS minimum on minimum spec hardware | P0 |
| Input latency | < 16ms from input to local response | P0 |
| Network latency perception | < 100ms round trip before player notices lag | P1 |
| Asset load stall | No visible geometry pop-in within 50 units | P1 |
| Memory usage | < 4 GB RAM on minimum spec | P1 |
| Asset cache | < 10 GB disk on default settings | P2 |

**Server Performance:**

| Metric | Requirement | Priority |
|--------|-------------|----------|
| Tick rate | 50 ticks/second sustained | P0 |
| Tick budget | < 20ms per tick for domains with up to 100 entities | P0 |
| Handoff latency | < 200ms for entity handoff between nodes | P1 |
| World graph write | < 10ms for single object state change | P1 |
| World graph spatial query | < 5ms for radius query returning up to 1,000 objects | P1 |
| Node spawn time | < 30 seconds from request to active | P2 |
| Ticker log append | < 1ms per entry | P0 |

**Scale:**

| Metric | Requirement |
|--------|-------------|
| Concurrent players per node | Up to 200 (target) |
| Objects per node domain | Up to 100,000 |
| Global concurrent players | Unbounded (scales with node count) |
| World graph objects | Up to 1 trillion (1e12) |
| Geographic regions | Up to 50 |

## 16.2 What Must Be Fast vs Slow

**Must be fast (happens every tick, in the critical path):**
- Physics integration
- Collision broad phase
- Entity behavior evaluation
- State change broadcasting
- Octree point query
- LOD tier assignment

**Can be slow (happens occasionally, off the critical path):**
- Asset compression and decompression (background thread)
- World graph shard rebalancing (happens during low traffic)
- Ticker log compaction (background process)
- Node split/merge (rare, brief impact)
- World graph cross-shard queries (rare — requires good world design)

**Must be parallelizable:**
- Physics integration (all bodies are independent — embarrassingly parallel)
- Entity behavior evaluation (entities are independent within a tick)
- Asset decompression (one per asset, multiple assets in parallel)
- World graph shard queries (each shard is independent)

**Must be sequential (cannot parallelize):**
- Collision resolution (pairs share state — must be resolved in order)
- World state broadcast after tick (must have complete tick results first)
- World graph writes within a single object's record (must preserve order)

---

# PART XVII: IMPLEMENTATION PHASES

## Phase 0: Foundation (Start Here)

**Goal**: A single node running locally, simulating an empty world, with one player connected.

What is built:
- The world graph (local instance, single shard)
- The node process (simulation tick, physics integration)
- The client (connects to node, renders an empty world)
- The WebSocket protocol (handshake, position update, simple action)
- The octree (in-memory, local to node)

What is NOT built yet:
- Multiple nodes
- The orchestration system
- Asset streaming (hardcode one object type)
- LOD system (hardcode single detail level)
- AI agents
- Building system

**Phase 0 definition of done**: A player can move around an empty terrain. Their position is simulated on the server. The client renders a flat plane. The server sends position updates 50 times per second. Frame rate is above 60 FPS.

## Phase 1: World Content

**Goal**: The world has content — terrain, objects, other players.

What is built:
- Procedural terrain generation
- Object types (rocks, trees, terrain features)
- Asset pipeline (compressed geometry, LOD tiers 0-4)
- Asset streaming (serve geometry on demand, cache locally)
- Multiple players (position updates for all visible players)
- Simple interaction (picking up objects)
- World graph persistence (objects survive node restart)

**Phase 1 definition of done**: Two players can see each other moving. They can pick up objects. Objects persist between sessions. The world looks like a real landscape.

## Phase 2: Multi-Node

**Goal**: The world spans multiple nodes. Players can cross node boundaries.

What is built:
- Orchestration system (domain map, node registry)
- Domain assignment and boundary handling
- Entity handoff protocol
- Adjacent node communication
- World graph sharding (at least 2 shards)

**Phase 2 definition of done**: Player A is connected to node 1. Player B is connected to node 2. They can see each other across the node boundary. Player A walks into node 2's domain and seamlessly transitions without any visible disruption.

## Phase 3: Full World Systems

**Goal**: The world is fully functional — building, agents, economy.

What is built:
- Building system (place, combine, destroy objects)
- Resource gathering
- Crafting recipes
- Inventory system
- AI agents (basic behavior trees)
- Machine objects (simple processors)
- The computer object (basic in-world computation)
- Ticker log (full audit and replay)

**Phase 3 definition of done**: Player can gather resources, craft tools, build structures, and interact with AI agents. A built machine processes inputs to produce outputs. Everything persists. History is auditable.

## Phase 4: Scale

**Goal**: The system runs with thousands of concurrent players across many nodes.

What is built:
- Load-based node splitting and merging
- Geographic distribution (at least 2 geographic regions)
- Edge gateways (player connection routing)
- Full orchestration (health monitoring, auto-scaling)
- Performance optimization pass against Phase 4 benchmarks
- The full observability stack (metrics, logs, alerts)

**Phase 4 definition of done**: 10,000 concurrent players in the world simultaneously. No single point of failure. Node loss is recovered automatically. Player experience degrades gracefully under extreme load rather than failing completely.

---

# APPENDIX A: ELEV8 FAILURE ANALYSIS

## A.1 Source Analysis

Analysis was conducted on the ELEV8 codebase — a Dreamworld prototype built under hackathon conditions. The following failures are documented not as criticism of the original engineers but as lessons that must be encoded into the design of NEXUS.

## A.2 Failure Catalog

### Failure A-01: The Three-Stack Problem

**What happened**: The project comprised three separate applications:
1. A vanilla JavaScript Three.js museum viewer
2. A custom physics graph visualizer
3. A Next.js / React application with AI chat

Each had its own build system, its own server, its own data model. They communicated through an Express proxy but had no shared state model.

**Why it failed**: When a change needed to affect all three (e.g., a new object placed in the museum should appear in the graph and the AI should know about it), it required three separate code paths, three separate data transformations, and three separate deployments. This was so expensive that cross-system features were simply not built.

**NEXUS resolution**: One data model. One state. The world graph is the single source of truth for all subsystems. A change written to the world graph is immediately visible to all subsystems that read from it. There is no "museum state" vs "graph state" vs "AI state" — there is only world state.

### Failure A-02: Custom Physics Engine

**What happened**: The team built a custom force-directed graph physics simulation from scratch in `3dGraphUniverse/src/physics.js`. This took an estimated 20+ hours of the 48-hour hackathon.

**Why it failed**: Custom physics engines have bugs that only appear under edge cases. The team encountered several: objects tunneling through each other, unstable simulation under many bodies, performance degradation with more than ~200 objects. These bugs were not fixable within the hackathon timeframe.

**NEXUS resolution**: Physics is specified abstractly (Part X of this document). The specification defines what physics must do, not how. The implementation phase selects from proven algorithms and implementations. The physics system is replaceable through an extension point. No physics logic is written from scratch unless no proven alternative exists.

### Failure A-03: Silent WebSocket Failure

**What happened**: The WebSocket layer received arbitrary JSON strings. AI output was piped directly into state — a LLM might return `{"position": [100, 200, 300]}` or it might return `"I apologize, I cannot..."`. Both arrived on the same channel. The second case caused silent crashes.

**Why it failed**: No schema validation. No error recovery path. No message typing. Any deviation from expected format caused invisible state corruption.

**NEXUS resolution**: Every message has a type byte, a version, and a schema. The receiver validates every message before processing. Invalid messages are rejected with an error message sent back to the sender. Critical state changes require acknowledgment — the sender knows whether its change was accepted or rejected. No silent failures.

### Failure A-04: Direct Three.js Mutation

**What happened**: Three.js objects (meshes, materials, lights) were mutated directly in response to network events:
```
onNetworkEvent -> object.position.set(x, y, z) -> Three.js renders new position
```
This bypassed React state entirely. The React state still held the old position. Database writes based on React state were therefore wrong.

**Why it failed**: Two sources of truth for the same fact (position). They diverged. Neither could be relied upon.

**NEXUS resolution**: The rendering layer is a pure consumer of world state. It reads the local world state and renders it. It never modifies world state directly. Player actions flow: input → action queue → server → authoritative state → local world state update → rendering. There is exactly one place where position is the truth: the world state. The renderer reads it.

### Failure A-05: No Spatial Index

**What happened**: The Postgres database stored object positions as plain float columns. "Find objects within 100 units of player" required:
```sql
SELECT * FROM objects WHERE
  sqrt((x - $1)^2 + (y - $2)^2 + (z - $3)^2) < $4
```
This is a full table scan with per-row computation. It could not scale.

**NEXUS resolution**: The world graph's spatial index is a first-class requirement (Part IX, Section 9.3). Every position is indexed. Spatial queries are logarithmic in the total number of objects. This is non-negotiable.

### Failure A-06: Monolithic Deployment

**What happened**: All three subsystems deployed together. A change to any subsystem required restarting all of them.

**NEXUS resolution**: Every subsystem is independently deployable. The contracts between subsystems (Part XV) define stable interfaces. A subsystem can be updated and redeployed without other subsystems knowing. The orchestration system handles container management independently per subsystem type.

---

# APPENDIX B: KNOWLEDGE-GRAPH CONCEPTS APPLIED

## B.1 Source

The knowledge-graph program (`programs/knowledge-graph`) describes a cognitive document system where:
- Documents are active nodes with embedded instructions
- Documents have 5D semantic positions
- The system maintains neighbor relationships automatically
- A ticker log records all access events
- Context files are auto-generated from access patterns

These concepts translate to the game engine world graph in the following ways.

## B.2 Translation Table

| Knowledge-Graph Concept | NEXUS Application |
|------------------------|-------------------|
| 5D semantic vector | Objects have a semantic vector (mass, energy, information-density, age, rarity) enabling non-spatial queries |
| Embedded prompt | Objects have embedded behavioral instructions for AI agents that interact with them |
| Automatic neighbor computation | When an object is added to the world graph, its spatial and semantic neighbors are computed automatically |
| Ticker log (access events) | World graph ticker log (all state changes) — same structure, different event types |
| Context file (auto-generated summary) | Each object has an auto-generated "environment summary" listing its neighbors and relationships |
| Self-navigating (AI reads neighbors) | Agents navigate the world by reading neighbors — no global queries needed for local behavior |
| Strict sub-program boundaries | Engine subsystems have hard contracts (Part XV) — same discipline, different domain |

## B.3 The 5D Object Vector

Every world object has a semantic vector with 5 dimensions:

```
OBJECT SEMANTIC VECTOR:
  [0] mass_index: 0.0 (weightless/information) to 1.0 (extremely dense/material)
  [1] energy_index: 0.0 (inert) to 1.0 (highly energetic/reactive)
  [2] information_index: 0.0 (simple/dumb) to 1.0 (complex/computational)
  [3] age_index: 0.0 (just created) to 1.0 (ancient/primordial)
  [4] rarity_index: 0.0 (ubiquitous) to 1.0 (unique)
```

This enables queries like:
- "Find the nearest highly energetic object" (energy_index > 0.8, spatial query)
- "Find everything computationally complex within 500 units" (information_index > 0.7, radius 500)
- "Find the oldest thing in this sector" (maximize age_index within sector)

These queries power AI agent behavior: an agent seeking energy finds energy sources without knowing their specific positions.

## B.4 Embedded Behavioral Instructions

Objects contain instructions that specify how they behave when an agent or player interacts with them. This is similar to the embedded_prompt in knowledge-graph nodes.

```
EXAMPLE OBJECT WITH EMBEDDED INSTRUCTIONS:
  type: energy_crystal
  semantic_vector: [0.2, 0.9, 0.3, 0.7, 0.6]  -- light, high energy, somewhat complex
  embedded_instructions:
    ON interact with action=HARVEST:
      IF agent.has_tool(ENERGY_HARVESTER):
        TRANSFER energy_amount to agent.energy_reserve
        REDUCE self.properties.energy by energy_amount
        IF self.properties.energy <= 0:
          EMIT event(DEPLETED)
      ELSE:
        RESPOND with "You need an energy harvester to collect this"

    ON agent_nearby within_radius=50:
      IF self.properties.energy > EMISSION_THRESHOLD:
        EMIT energy_pulse affecting agents within 20 units
```

---

# APPENDIX C: OPEN ARCHITECTURAL DECISIONS

These are the decisions that must be made before Phase 0 can begin. Each is an ADR (Architecture Decision Record) stub. They will be moved to `programs/game_engine/_planning/adr/` when expanded.

| ADR | Decision | Status | Options |
|-----|----------|--------|---------|
| ADR-001 | Sector size in world units | Open | 500, 1000, 2000 units |
| ADR-002 | World graph spatial index type | Open | R-tree, octree, spatial hash, Hilbert |
| ADR-003 | Physics integration algorithm | Open | Semi-implicit Euler, Verlet, constraint-based |
| ADR-004 | Collision detection library vs custom | Open | Custom (specified above), proven library |
| ADR-005 | GPU texture compression format | Open | BC7, ASTC, ETC2, format selector |
| ADR-006 | Binary message serialization format | Open | Flatbuffers, protobuf, custom, Cap'n Proto |
| ADR-007 | Edge gateway architecture | Open | Proxy mode, redirect mode, hybrid |
| ADR-008 | Ticker log storage | Open | Append-only file, Kafka-style log, time-series DB |
| ADR-009 | World graph replication | Open | CRDTs, vector clocks, last-write-wins, consensus |
| ADR-010 | In-world computer language | Open | Custom restricted language, Lua subset, WASM sandbox |
| ADR-011 | Geographic region boundary strategy | Open | Fixed continents, player-shaped, emergent |
| ADR-012 | Asset compression algorithm | Open | Draco (meshes), Basis Universal (textures), custom |
| ADR-013 | Behavior AI evaluation order | Open | Sequential, priority-sorted, parallel evaluated |
| ADR-014 | World seed algorithm | Open | Simplex noise, OpenSimplex, domain-warped noise |

---

# APPENDIX D: DREAMWORLD REQUIREMENTS MAP

This appendix maps each requirement from `programs/dreamworld/prd.txt` to the relevant section of this PRD.

| Dreamworld Requirement | Addressed In |
|-----------------------|-------------|
| 3D spatial internet | Part I.1, Part III |
| Persistent objects | Part IX (World Graph), Part VII (Asset Pipeline) |
| Millions of concurrent users | Part XIV (Platform), Part XVI (Performance Contracts) |
| AI agents in the world | Part XII (Agent System) |
| User builds and creates | Part XIII (Building System) |
| Natural language world creation | Appendix C, ADR-010 (in-world computation) |
| Universal graph-based multiverse | Part IX (World Graph), Appendix B |
| Physics environment | Part X (Simulation) |
| Infinite world traversal | Part III (World Architecture), Part VII (Asset Pipeline) |
| Browser-based (Three.js/WebGPU) | Part V (Local Engine) — technology choice deferred to ADR |
| Node.js gateway | Part XIV (Edge Gateway) |
| PostgreSQL spatial indexing | Part IX.3 (Spatial Indexing) — technology choice open |
| WebSocket real-time sync | Part VIII (Network Protocol) |
| Token economy | Deferred — not in engine scope, in game-logic layer above engine |
| Telegram bot interface | Deferred — separate interface, not in engine scope |

---

*End of NEXUS Game Engine PRD v0.1*

*Next actions:*
*1. Expand all open ADRs in `programs/game_engine/_planning/adr/`*
*2. Write shared contracts for all 6 subsystems listed in Part XV*
*3. Run spec-review on each section*
*4. Resolve ADR-001 through ADR-005 (blocking Phase 0)*
*5. Create build order in `programs/game_engine/_planning/roadmap.md`*
