---
name: world-graph-contract
status: accepted
version: 0.1
published_by: orchestration layer
consumed_by: world/node-manager/ (portal traversal), engine/ (constellation rendering, map), edge-gateway (world routing)
---

# World Graph Contract

The world graph is the topology of the Dreamworld multiverse. Each node in the graph is a world. Each edge is a connection between worlds (a portal, a spatial adjacency, or a parent-child nesting). The graph is fractal: worlds contain subworlds contain subworlds infinitely deep. The world graph answers: "What worlds exist, how are they connected, and how do I get from here to there?"

This is NOT the node-registry. The node-registry maps spatial domains to running server processes within a single world. The world graph maps worlds to each other across the entire multiverse.

```
node-registry:   "position P is owned by server process N"    (within one world)
world-graph:     "world A connects to world B via portal X"   (across all worlds)
```

---

## What This Contract Provides

### READ operations

`get_world(world_id: uint64)` → world_record | NOT_FOUND
  - Returns the full world record for a given ID

`get_world_by_slug(slug: string)` → world_record | NOT_FOUND
  - Lookup by human-readable slug (URL-like: "roan/house", "pvp/arena-1")

`get_children(world_id: uint64)` → list of world_record
  - Returns all worlds whose parent_id == world_id (direct children only)
  - Sorted by created_at ascending

`get_ancestors(world_id: uint64)` → list of world_record
  - Returns the chain from this world to the root: [parent, grandparent, ..., root]
  - Used for breadcrumb navigation and permission inheritance

`get_edges(world_id: uint64)` → list of world_edge
  - Returns all edges (connections) involving this world
  - Includes parent-child edges AND portal edges AND spatial adjacency edges

`get_nearby_worlds(world_id: uint64, max_depth: uint8)` → list of (world_record, graph_distance: float32)
  - BFS traversal up to max_depth edges from the source world
  - Returns worlds sorted by cumulative edge weight (graph distance)
  - Used by the client to render the constellation (nearby worlds visible in the sky)

`get_portals_in_world(world_id: uint64)` → list of portal_record
  - Returns all portal objects placed in a given world
  - Used by the renderer to show portal destinations and by the server for traversal

`find_path(from_world_id: uint64, to_world_id: uint64, strategy: PathStrategy)` → list of world_edge | NO_PATH
  - Pathfinding through the world graph
  - Strategy: SHORTEST (fewest edges), CHEAPEST (lowest cumulative weight), SAFEST (avoids PvP worlds)
  - Returns ordered list of edges to traverse
  - Returns NO_PATH if worlds are in disconnected subgraphs with no portal link

`search_worlds(query: string, filters: WorldSearchFilters)` → list of world_record
  - Full-text + semantic search across world names, descriptions, tags
  - Filters: owner_id, world_type, min_player_count, max_player_count, is_public, game_mode

### WRITE operations

`create_world(config: WorldCreateRequest)` → world_record | create_error
  - Creates a new world node in the graph
  - If parent_id is set, creates as a child (subworld)
  - If parent_id is null, creates as a root-level world (floating island in constellation)
  - Validates: owner has permission, world limit not exceeded

`update_world(world_id: uint64, updates: WorldUpdateRequest)` → world_record | update_error
  - Updates world metadata (name, description, config, visibility)
  - Does NOT modify edges — use edge operations for that

`delete_world(world_id: uint64)` → accepted | delete_error
  - Soft-delete: marks as deleted, stops spawning processes
  - Cascading: all child worlds (subworlds) are also soft-deleted
  - All portals pointing to this world become inactive (render as broken portal)
  - Recoverable for 30 days, then hard-deleted

`create_edge(edge: WorldEdgeCreateRequest)` → world_edge | edge_error
  - Creates a connection between two worlds
  - Edge type must be specified (PORTAL, SPATIAL_ADJACENCY, PARENT_CHILD)
  - PARENT_CHILD edges are created automatically by create_world when parent_id is set

`delete_edge(edge_id: uint64)` → accepted | edge_error
  - Removes a connection between worlds
  - If PORTAL type: the portal objects in both worlds become inactive
  - PARENT_CHILD edges cannot be deleted (delete the child world instead)

`create_portal(portal: PortalCreateRequest)` → portal_record | portal_error
  - Creates a portal object in source_world at source_position pointing to target_world at target_position
  - Also creates the corresponding world_edge of type PORTAL
  - Optionally bidirectional: creates a return portal in target_world

---

## Data Shapes

### world_record

```
WORLD_RECORD:
  IDENTITY:
    world_id:        uint64     — globally unique, never reused
    slug:            string     — human-readable URL path ("roan/house", "pvp/arena-1")
    name:            string     — display name ("Roan's Island", "PvP Arena #1")
    description:     string     — short description for map/search
    owner_id:        uint64     — player_id of the world creator
    created_at:      uint64     — Unix timestamp ms
    updated_at:      uint64     — Unix timestamp ms

  GRAPH:
    parent_id:       uint64 | null   — parent world (null = root-level world)
    depth:           uint16          — distance from root (0 = root-level)
    subworld_of_object: uint64 | null — if this world is inside an object, the object's ID
    edge_count:      uint16          — number of edges (portals + adjacencies)

  TYPE:
    world_type:      uint8      — 0=sandbox, 1=survival, 2=pvp, 3=story, 4=learning, 5=shop
    game_mode:       uint8      — 0=peaceful, 1=survival, 2=pvp_enabled, 3=creative
    is_public:       bool       — visible in constellation and search
    is_template:     bool       — available as template for new users

  CONFIG:
    physics_config:  WorldPhysicsConfig   — gravity, damping, time_scale (from simulation spec)
    terrain_type:    uint8      — 0=flat, 1=heightmap, 2=procedural, 3=none (floating structure)
    atmosphere:      AtmosphereConfig     — sky color, fog, weather
    spawn_position:  Vec3f64    — where players appear when entering this world
    spawn_orientation: Quat32   — which direction they face
    max_objects:     uint32     — object limit for this world (resource constraint)
    max_players:     uint32     — concurrent player limit (0 = unlimited)
    seed:            uint64     — procedural generation seed (ADR-014)

  STATUS:
    state:           uint8      — 0=sleeping, 1=active (process running), 2=deleted
    player_count:    uint32     — current online players (0 when sleeping)
    object_count:    uint32     — total persistent objects
    last_active_at:  uint64     — when the last player left (used for cleanup)

  CONSTELLATION:
    constellation_position: Vec3f64   — position in the constellation sky view
    constellation_radius:   float32   — visual size in constellation (based on object_count / activity)
    constellation_color:    uint32    — RGBA display color in constellation
    atmosphere_halo:        bool      — render atmosphere glow in constellation view
```

### world_edge

```
WORLD_EDGE:
  edge_id:         uint64
  source_world_id: uint64
  target_world_id: uint64
  edge_type:       uint8       — 0=PORTAL, 1=SPATIAL_ADJACENCY, 2=PARENT_CHILD
  weight:          float32     — graph distance (lower = closer in constellation)
  bidirectional:   bool        — if true, traversable both ways

  -- PORTAL-specific fields (only when edge_type == 0):
  source_portal_id:  uint64    — portal object ID in source world
  target_portal_id:  uint64    — portal object ID in target world (if bidirectional)
  source_position:   Vec3f64   — portal position in source world
  target_position:   Vec3f64   — arrival position in target world

  -- SPATIAL_ADJACENCY-specific (only when edge_type == 1):
  merge_distance:    float32   — if weight < merge_distance, worlds share continuous terrain
  boundary_axis:     uint8     — 0=X, 1=Y, 2=Z — which axis they share a boundary on
```

### portal_record

```
PORTAL_RECORD:
  portal_id:          uint64      — also an object_id in the world's object graph
  world_id:           uint64      — world this portal exists in
  position:           Vec3f64     — position in the world
  orientation:        Quat32      — facing direction
  target_world_id:    uint64      — destination world
  target_position:    Vec3f64     — arrival position in destination
  target_orientation: Quat32      — arrival facing direction
  edge_id:            uint64      — references the world_edge

  visual:
    portal_style:     uint8       — 0=doorway, 1=archway, 2=vortex, 3=mirror, 4=invisible_trigger
    preview_enabled:  bool        — render a live preview of the destination through the portal
    label:            string      — display name shown above portal ("Roan's Island →")

  state:
    is_active:        bool        — false if target world is deleted or edge removed
    is_locked:        bool        — requires key/permission to traverse
    lock_type:        uint8       — 0=none, 1=owner_only, 2=group, 3=token_cost, 4=item_key
    lock_param:       string      — group_id, token_amount, or item_type depending on lock_type

  cooldown_ms:        uint32      — minimum time between traversals (prevent spam, default 1000)
```

### subworld_link

```
SUBWORLD_LINK:
  object_id:        uint64      — the object you "enter" to access the subworld
  subworld_id:      uint64      — the world_id of the nested world
  entry_trigger:    uint8       — 0=walk_into, 1=interact (press E), 2=zoom_in
  entry_position:   Vec3f64     — where you appear inside the subworld
  exit_position:    Vec3f64     — where you reappear in the parent world on exit
  scale_factor:     float32     — size ratio (parent units / subworld units)
                                -- e.g., 0.01 means the subworld is 100x larger inside
```

---

## Portal Traversal Protocol

When a player walks into a portal, the following sequence occurs:

```
PORTAL TRAVERSAL:

  [1] CLIENT detects collision with portal trigger volume
      → sends PORTAL_TRAVERSE request (C→S, Protobuf)
      → payload: portal_id, player's current position + velocity

  [2] CURRENT WORLD SERVER validates:
      → portal exists and is active
      → player is within trigger distance
      → cooldown not active
      → lock check passes (permission, token balance, item key)
      IF validation fails → send ERROR to client, abort

  [3] CURRENT WORLD SERVER queries world-graph:
      → get_world(portal.target_world_id) → target world_record
      → IF target world is sleeping → request orchestration to spawn it
      → WAIT until target world process is active (timeout: 10 seconds)

  [4] CURRENT WORLD SERVER issues NODE_TRANSFER to client:
      → new_node_address: target world's server address
      → transfer_token: single-use token encoding:
          - player_id
          - target_world_id
          - target_position (arrival coordinates)
          - target_orientation
          - expiry timestamp (10 seconds)

  [5] CLIENT receives NODE_TRANSFER:
      → begins transition animation (fade, whoosh, portal vortex)
      → opens new WebSocket to target world address
      → sends HANDSHAKE with auth_token = transfer_token

  [6] TARGET WORLD SERVER validates transfer_token:
      → checks player_id, expiry, world_id match
      → spawns player entity at target_position with target_orientation
      → sends HANDSHAKE_RESPONSE (ACCEPTED) + full STATE_SNAPSHOT
      → broadcasts PLAYER_JOINED to other clients in target world

  [7] CLIENT receives HANDSHAKE_RESPONSE + STATE_SNAPSHOT:
      → renders target world
      → closes old WebSocket connection
      → transition animation completes (fade in)

  [8] OLD WORLD SERVER detects client disconnect:
      → broadcasts PLAYER_LEFT to remaining clients
      → persists player's last state
      → if no players remain → begin drain → sleep
```

### Subworld Entry (same protocol, different trigger)

Entering a subworld (walking inside an object) follows the same protocol. The difference:

- The trigger is the object's `subworld_link.entry_trigger` instead of a portal collision
- The `scale_factor` is sent in the transfer_token so the client can adjust camera/rendering
- Exit from the subworld is via a special "exit portal" at the subworld's designated exit point
- The exit portal always targets the parent world at `subworld_link.exit_position`

---

## Constellation Rendering

The client uses the world graph to render the constellation — the sky view of nearby worlds.

```
CONSTELLATION RENDER:

  [1] Client calls get_nearby_worlds(current_world_id, max_depth=3)
      → receives list of (world_record, graph_distance) within 3 edges

  [2] For each nearby world:
      → position in sky = world_record.constellation_position (relative to current world)
      → visual size = constellation_radius / graph_distance
      → color = constellation_color
      → glow = atmosphere_halo
      → player activity = player_count (pulsing intensity)

  [3] Render each world as:
      → Distant (graph_distance > 2.0): point light with halo
      → Medium (0.5 < graph_distance <= 2.0): small sphere with atmosphere
      → Near (graph_distance <= 0.5): LOD terrain preview visible

  [4] Portal beams: for each PORTAL edge from current world, render a
      glowing line from portal position toward the target world's
      constellation position. Visual cue: "that portal goes there."

  [5] Route overlay: if player has an active route (from map/route planning),
      highlight the path through the constellation as a brighter line.
```

---

## World Lifecycle Integration

The world graph integrates with the existing node-manager lifecycle:

```
WORLD SLEEPING → ACTIVE:
  Trigger: first player enters (via portal, teleport, or direct connect)
  1. Orchestration receives "spawn world" request
  2. Orchestration reads world_record.config from world graph DB
  3. Spawns Rust world process with config
  4. Process loads persistent state from world graph DB (objects, terrain)
  5. Process registers with node-registry
  6. World graph updates: state = active, player_count = 1

WORLD ACTIVE → SLEEPING:
  Trigger: last player leaves (disconnect or portal out)
  1. Node-manager enters DRAIN mode
  2. Flushes all state to world graph DB
  3. Deregisters from node-registry
  4. Process exits
  5. World graph updates: state = sleeping, last_active_at = now()

WORLD NEVER VISITED:
  state = sleeping, player_count = 0, last_active_at = null
  No process, no memory, no CPU. Just a row in the database.
```

---

## World Creation Flow

```
CREATE WORLD (from in-game computer):

  [1] Player opens "Create World" on computer OS
      → selects: name, world_type, game_mode, terrain_type
      → optionally selects parent world (creates as subworld)
      → optionally selects template

  [2] Client sends WorldCreateRequest as Packet to gateway
      → gateway routes to world-graph service

  [3] World-graph service validates:
      → owner has permission (world limit not exceeded)
      → slug is unique
      → parent exists (if parent_id set)
      → template exists (if template_id set)

  [4] World-graph service creates:
      → world_record in database
      → PARENT_CHILD edge if parent_id set
      → copies template config if template_id set
      → assigns constellation_position:
          - If has parent: near parent's position + small offset
          - If root: random position in constellation with minimum spacing

  [5] Returns world_record to client
      → client shows "World created!" with option to enter

  [6] Player can now place a portal in their current world pointing
      to the new world (or teleport directly from the map)
```

---

## Subworld-Inside-Object Creation

```
CREATE SUBWORLD IN OBJECT:

  [1] Player selects an object in placement mode
      → "Create interior" option available for objects tagged as enterable

  [2] System creates a new world with:
      → parent_id = current world
      → subworld_of_object = object_id
      → spawn_position = center of subworld

  [3] System creates a subworld_link:
      → object_id = the selected object
      → subworld_id = new world
      → entry_trigger = walk_into or interact
      → scale_factor = configurable (default 1.0)

  [4] Player can now enter the object and build the interior
      → the subworld has its own physics config, lighting, atmosphere
      → completely independent from the parent world
```

---

## Constants

```
MAX_WORLDS_PER_USER_FREE     = 3
MAX_WORLDS_PER_USER_PAID     = 50
MAX_SUBWORLD_DEPTH           = 10        -- prevent infinite nesting
MAX_PORTALS_PER_WORLD        = 100
MAX_EDGES_PER_WORLD          = 200
PORTAL_TRIGGER_RADIUS        = 2.0       -- units, collision trigger distance
PORTAL_COOLDOWN_DEFAULT      = 1000      -- ms between traversals
WORLD_SPAWN_TIMEOUT          = 10000     -- ms, max wait for sleeping world to wake
CONSTELLATION_QUERY_DEPTH    = 3         -- max edge depth for nearby world query
CONSTELLATION_MERGE_WEIGHT   = 0.1       -- edge weight below which worlds share terrain
MIN_CONSTELLATION_SPACING    = 50.0      -- units, minimum distance between root worlds
WORLD_SOFT_DELETE_RETENTION  = 30        -- days before hard delete
```

---

## New Schemas Required

This contract requires two new Protobuf schemas in `shared/schemas/`:

### portal_traverse.proto (C→S, type 0x0104)
```
PORTAL_TRAVERSE:
  portal_id:    uint64
  position:     Vec3f64      -- player's current position (server validates proximity)
  velocity:     Vec3f32      -- for momentum preservation through portal
```

### world_info.proto (S→C, type 0x0105)
```
WORLD_INFO:
  world_id:     uint64
  name:         string
  world_type:   uint8
  game_mode:    uint8
  player_count: uint32
  constellation_position: Vec3f64
  constellation_radius:   float32
  constellation_color:    uint32
```

---

## What This Contract Does NOT Provide

- Spatial domains within a world (that is node-registry-contract)
- Physics simulation (that is simulation-contract)
- Object persistence within a world (that is world-state-contract)
- Asset storage (that is asset-store-contract)
- Player authentication (that is player-session-contract)
- World process management (that is orchestration — Layer 5)
