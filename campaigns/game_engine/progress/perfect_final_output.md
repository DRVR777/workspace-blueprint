# The Perfect Final Output
## What The Finished System Is — Completely Abstract

*No code. No language. No framework names. Pure behavioral description.*
*This is the north star. Every decision in the PRD is measured against this document.*

---

## Preface

There is a universe. It existed before you arrived, and it will exist after you leave. It is three-dimensional, persistent, and shared. It is made of objects and relationships and physics and time. You enter it through a window on your machine. You leave your mark on it. Others do the same. The marks persist.

This document describes what that universe feels like from every vantage point: the player who inhabits it, the machine that renders it, the server that simulates it, and the database that remembers it.

---

## Part I: What The Player Experiences

### The Moment of Entry

When you enter the world, you begin as a single point in three-dimensional space. You have a position — a triplet of coordinates that places you somewhere in the universe. That position is yours. It persists between sessions. You enter where you left.

Around that position, reality materializes. Not all at once — the world assembles itself in concentric rings of detail. What is closest to you is most real: fully formed, textured, physically present. What is farther away is present but simplified — outlines, shapes, approximations. What is very far is barely a suggestion — a color gradient on the horizon, a distant point of light, a blurred mass.

You do not experience this assembly as loading. It is seamless. The world flows into focus as you move through it.

### Movement and Space

You move through the world freely. You can walk, run, fly, or teleport — the engine does not prescribe your mode of movement. What it guarantees is that wherever you go, the world is there. There are no edges. There are no loading screens. There are no invisible walls.

As you move, the sphere of reality moves with you. Things behind you simplify. Things ahead of you sharpen. This happens at the rate of your movement, not faster, not slower.

The world has terrain — elevation, water, open plains, dense structures. Terrain is a special kind of object: it covers enormous volumes of space but costs very little to store and transmit, because it is described mathematically rather than as a collection of discrete shapes.

### Objects and Their Reality

Objects exist in the world independently of you. A rock is there whether or not you are near it. A building exists even when no player is within kilometers of it. The server knows about these objects and maintains their state. When you enter their vicinity, your machine receives information about them and renders them.

Objects have properties: position, orientation, shape, material, state, age, ownership, relationships. Some properties are visible. Some are hidden. Some can be changed. Some cannot.

The shapes of objects are transmitted to your machine once and then stored locally. The next time you encounter that object — or any object of the same type — your machine already knows what it looks like. The server only needs to tell you where it is and what state it is in. This means that as you traverse the world, the amount of data flowing to your machine decreases over time, not increases. The world becomes cheaper to render as you explore it.

### Other Presences

Other players appear around you as presences. They have bodies — representations in the world. Their movements are transmitted to you in real time via the server. You see them move. They see you move. The transmission is fast enough that it feels simultaneous.

AI entities also inhabit the world. They move, interact with objects, respond to players. They are indistinguishable in appearance from other kinds of entities. Their behavior is governed by instructions embedded in their definitions.

### Building and Creating

You can interact with the world physically. You can pick up objects, place them, combine them, destroy them. These actions persist — the world remembers what you did.

You can create new objects by combining existing ones. The rules for combination are defined by the world's simulation layer. Some combinations produce new objects. Some produce energy. Some produce nothing.

The most powerful thing you can create is a functional machine — a device that processes information and produces outputs according to rules. These machines exist physically in the world. Other players can interact with them. They can be connected to each other, forming networks of computation within the universe.

### The Edge of Perception

When you look toward the horizon, you see the world simplify gradually. Distant objects become silhouettes. Very distant objects become mere indicators — icons or points of light that tell you something exists in that direction. Beyond a certain distance, the world does not exist in your view at all — not because it isn't there, but because the information would be meaningless at that scale.

When another player is very far away, you see a simple marker. When they move closer, their form gains detail. This is not a binary switch — it is a continuous gradient from formless to fully real.

---

## Part II: What The Local Machine Does

### The Rendering Loop

The local machine runs a continuous loop at high frequency. Each iteration of the loop produces one frame — one still image of the world as it exists at that instant, from the player's viewpoint. These frames play back so fast that they appear to be continuous motion.

Each frame requires the machine to:
- Know where the player is
- Know the positions and states of all nearby objects
- Know what those objects look like (their geometry)
- Determine which objects are visible from the current viewpoint
- Sort them by distance and type
- Render them, near objects in full detail, far objects in reduced detail
- Apply lighting, shadows, and atmosphere
- Composite the result and display it

The rendering loop does not wait for the network. Network updates arrive asynchronously and are merged into the world state. If no update arrives for a moment, the local machine uses what it last knew, extrapolating movement and physics locally. When the authoritative update arrives, any discrepancy is corrected smoothly, not abruptly.

### Local Physics Prediction

The local machine runs a physics simulation — not the authoritative one (that runs on the server) but a prediction. It predicts how objects will move based on the last known state and the laws of physics. Most of the time, the prediction is exactly right. When the server says something different happened, the local machine corrects itself.

This approach means that the local experience always feels responsive. You press a button and something happens instantly, locally. The server confirms or corrects a fraction of a second later.

### The Asset Cache

The local machine maintains a persistent store of object geometry — shapes, textures, materials. This store grows over time as the player encounters new objects. It is organized by object type, not by location. If the same tree type appears in ten different places, the local machine only stores its geometry once.

The cache has a size limit. When it is full, the least recently used assets are removed. The machine will re-request them if the player encounters them again.

The cache persists between sessions. When you re-enter the world, you still have all the geometry from your previous session. The world is immediately more renderable than it was on your first entry.

### The Visibility System

Every frame, the local machine determines what is visible from the current viewpoint. This is not a brute-force check of all known objects — it uses spatial data structures to quickly eliminate entire regions of space that cannot possibly be visible. What remains is a small set of candidate objects that are then individually checked.

The result is a sorted list of visible objects, each tagged with its distance from the player. This list is handed to the LOD system.

### The LOD System

The LOD system assigns a detail level to each visible object based on its distance. It does this by consulting a set of distance thresholds. Below a threshold, an object gets its highest-detail geometry. Above it, a progressively simpler version. The simplest version is just a rough shape or even a single colored point.

Transitions between detail levels are smooth. The system does not abruptly swap geometry — it blends between levels, so the player never notices the change.

When the LOD system decides an object needs a geometry tier it doesn't have locally, it queues a request to the asset streaming system. The streaming system fetches it from the server in the background. Until the geometry arrives, the object uses the next simplest version it has.

---

## Part III: What A Single Server Node Does

### The Node's Identity

A node is a running process on a server. It has:
- A spatial domain — a three-dimensional region of the universe it is responsible for
- A connection to the world graph database — where all objects are stored
- A connection to the message bus — where it communicates with other nodes and with players
- A set of connected clients — the players whose 3D positions are within its domain

A node does not share its domain with other nodes. Each point in space is owned by exactly one node at any time. When a node goes offline, another node absorbs its domain.

### What A Node Computes

Every tick (a fixed interval, many times per second), a node computes the new state of everything in its domain:
- All objects: apply physics, gravity, momentum, collisions
- All entities: apply AI behaviors, movement rules
- All interactions: resolve any player actions that occurred since the last tick
- All events: fire any scheduled events

After computing, the node broadcasts the changes to all connected clients and writes the changes to the world graph database.

### Client Management

When a player's position approaches the boundary of a node's domain, the node initiates a handoff sequence with the adjacent node. The two nodes negotiate: the adjacent node prepares to accept the player. The player's client is given the address of the new node. The connection transitions. During this transition, the player experiences nothing unusual — the world continues rendering from local state while the network connection shifts.

A node can also split — if its domain becomes too crowded, it negotiates with the orchestration system to subdivide its domain, spawning a child node to handle part of it. Similarly, adjacent nearly-empty nodes can merge.

### What A Node Serves

A node serves two kinds of data to its connected clients:

**Continuous stream (metadata):**
- Position updates for all visible entities
- State changes for all visible objects
- Events occurring in the domain
- Chat and communication signals

**On-demand (assets):**
- Geometry for objects the client has not yet cached
- Texture data
- Sound definitions
- Object behavioral specifications

Asset serving is low-priority compared to metadata. Metadata always goes first. Assets wait until there is bandwidth headroom.

---

## Part IV: What The Server Mesh Is

### Many Nodes, One World

The universe is large enough that no single machine could simulate it alone. Instead, thousands of nodes run across hundreds or thousands of servers, each owning a piece of space. From the outside, this looks like one world. From the inside, each node only knows its piece deeply and knows the rest shallowly.

Nodes are not statically assigned to regions. The assignment is dynamic. When a region becomes busy, it gets more nodes. When a region empties, its nodes wind down or migrate.

### The World Graph As Shared Reality

The world graph is the single source of truth for everything that exists in the universe. Every object, every relationship, every event history — all of it lives in the world graph. Nodes read from it to discover what exists in their domain. Nodes write to it when things change.

The world graph is distributed across many machines. It is not a single database — it is a cluster of databases that together maintain a consistent picture of the world. Writes are fast because they are local to the relevant region. Reads are fast because spatial queries only touch the relevant partition.

### Cross-Node Communication

When an object crosses a node boundary — a player moving from one domain to the next, a projectile flying through the air — the nodes communicate directly. They use a message-passing system that is fast enough to be invisible to the player.

The world graph records the handoff. The object's record is updated atomically: the old domain relinquishes it; the new domain claims it. There is never a moment when the object is in both or neither.

### The Orchestration Layer

Above the nodes is an orchestration system. It maintains the map of which node owns which region. It monitors node health. It spawns new nodes when a region needs more capacity. It terminates idle nodes. It ensures that every point in the universe is always covered by exactly one running node.

The orchestration layer does not simulate the world — that is the nodes' job. It only manages the nodes themselves.

---

## Part V: What The World Graph Is

### The Graph Structure

The world graph is a connected structure in which every object is a node and every relationship is an edge. Relationships include: spatial proximity, ownership, construction (this object was built using that object), causality (this event caused that object to change state), and semantic similarity.

The graph is spatial — every node has a three-dimensional position. Queries can be spatial (find everything within this sphere) or semantic (find everything related to this concept) or relational (find everything connected to this object).

### Object Records

Every object in the world has a record in the world graph. The record contains:
- A unique identity that persists forever
- The three-dimensional position of the object
- The current state of the object (its properties and their values)
- The geometric description of the object (or a reference to it in the asset store)
- The history of state changes (the ticker log)
- The list of neighbors in the graph (both spatial neighbors and relational neighbors)
- Embedded behavioral instructions (what this object does when interacted with)

### The Ticker Log

Every change to the world is recorded as an entry in the ticker log. An entry contains: when the change happened, what changed, who or what caused it, and what the previous state was. The ticker log is append-only — entries are never modified or deleted.

This makes the world auditable. Any past state of the world can be reconstructed by replaying the ticker log from the beginning. Any dispute about what happened can be resolved by consulting the log.

### Self-Navigating Nodes

Objects in the world graph know their neighbors. When a new object is added to the world, the world graph automatically computes its nearest neighbors in all relevant dimensions (spatial, semantic, relational) and records those relationships. When an object moves, its neighbor relationships are updated.

This means that a query like "what is near this object" is answered by reading the object's own record, not by scanning the entire graph. The graph is self-indexing.

---

## Part VI: The Economy of Scale

### Why It Can Handle Millions of Players

The system is designed so that each component handles only a bounded amount of work, regardless of how many players exist in total. A node handles only the players in its region. The world graph handles only queries for its partition. The asset cache handles only what has been seen locally.

As the number of players grows, more nodes are spawned, more graph partitions are created, more servers are added. The work is spread proportionally. No single point bears disproportionate load.

### Why Objects Are Almost Free

The most expensive thing in a 3D world is rendering geometry. The system minimizes this in two ways:

First, geometry is cached locally. Once your machine has the shape of a tree, it can render a million trees of that type with almost no additional data cost.

Second, geometry is tiered by detail. When an object is far away, you render a version with very few polygons. The full geometry is only needed when the player is close — and at that point, the object probably occupies a significant portion of the screen, justifying the cost.

These two properties together mean that the world can have an enormous number of objects without the cost per player growing proportionally.

### Why Servers Only Send What Changes

The server does not send the state of the whole world every tick. It sends only what changed since the last tick. Most objects, most of the time, do not change. A rock sitting on a hillside sends nothing. A moving player sends a few bytes per tick. An explosion sends a burst.

This means that a node handling a dense region with many interactions is busier than a node handling an empty region — but the empty region costs almost nothing. The system scales with actual activity, not theoretical maximum activity.

---

## Part VII: What Success Looks Like From Every Angle

### From the player's perspective:
You join a living, infinite world. You can go anywhere. You can build anything. The world remembers you. Others share it with you. It feels real.

### From the operator's perspective:
The world runs on a cluster of servers that expands automatically as the player base grows. Cost scales with usage. Regions with activity cost more; empty regions cost almost nothing. You can inspect the state of any part of the world at any time. You can replay any event in history.

### From the developer's perspective:
Every subsystem is isolated behind a contract. You can replace the physics implementation without touching the rendering system. You can add new object types without modifying the world graph. You can add new node behaviors without redeploying the entire cluster. The system is designed so that the thing you want to change is the only thing you have to change.

### From the object's perspective:
Every object in the world is a complete self-describing entity. It knows what it is, where it is, what it does, what it is related to, and what has happened to it. It is never defined by a reference to some external table — it carries its identity with it. It knows its neighbors. When an agent interacts with it, it knows how to respond.

### From the AI agent's perspective:
An agent perceives the world through the same mechanisms as a player: a spatial query that returns nearby objects and their properties. It acts on the world through the same interfaces: place, pick up, combine, communicate. It has instructions embedded in its definition. Those instructions tell it what to do in each situation it encounters. The agent is not a special case — it is an entity like any other entity, with a different set of instructions.

---

## Closing Statement

The finished system is not a game engine in the traditional sense. It is a spatial computing substrate — infrastructure for a category of experience that does not currently exist at this scale, this fidelity, or this interactivity. It is designed to be the ground truth of a world, not a rendering library or a networking framework.

Every design decision made in its construction should be measured against this document. If a decision makes the above description possible, it is correct. If it makes the above description less possible, it is wrong.

The PRD exists to operationalize this vision. It takes every paragraph above and asks: what exactly must be true, in what order, for this to exist?

*The answer to that question is the engine.*
