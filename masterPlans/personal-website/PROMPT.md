# PROMPT — Personal Website World

## What To Build

Design a complete architecture plan for **roancurtis.com** — a personal website that isn't a website. It's a portal into a persistent 3D universe running on the user's machine (and optionally across a decentralized network of nodes). When someone visits roancurtis.com, they don't see a flat page — they enter a world.

---

## The Core Concept

Every user who signs up gets a **personal world** — a 3D space they inhabit, customize, and use as their digital home. Inside that world they can:

1. **Store files** — files are physical objects in the world (books on shelves, documents on desks, media on screens). Not metaphorical — actual 3D objects backed by real storage.

2. **Accumulate in-game items** — as users interact with the platform, build things, contribute content, complete challenges, or participate in commerce, they earn items. Items have real utility inside the world (tools, furniture, compute upgrades, cosmetics, vehicles).

3. **Purchase real-world assets** — the in-game economy bridges to real commerce. Users can buy physical goods, digital products, services, and subscriptions from businesses that have storefronts inside the world. The storefront is a 3D space inside the universe, not a web page.

4. **Access a universe inside their computer** — the world runs locally first (like a game client), connecting to a network of nodes for persistence, multiplayer, and shared state. Your computer IS your node. Your world IS your home server.

---

## Architecture Foundations — Pull From These Systems

This plan should synthesize and build on top of the following existing systems in this workspace. Read them thoroughly before designing:

### From Dreamworld PRD (`programs/dreamworld/PRD.md`)
- The **Universal Packet Protocol** — every interaction is a self-describing Packet with Header + Body
- The **World System** — graph-based multiverse where nodes are worlds, edges connect them
- The **In-Game Computer** — any surface is a screen, computers backed by real VPS containers
- The **Camera Spine** — continuous scroll zoom from first-person to bird's eye
- The **Agent System** — hierarchical AI agents (orchestrator → supervisors → workers → semantic)
- The **Knowledge Graph memory layers** — episodic (raw events), deep (patterns), shared (community)
- The **Economy** — tokens pegged to real compute costs, adding is free, extracting costs
- The **Template House** — every user starts with a customizable floating island + house + office

### From Knowledge Graph (`programs/knowledge-graph/`)
- **Real embeddings** (Gemini 3072D vectors) for semantic positioning of all content
- **Entity extraction** on every piece of content → typed edges between documents sharing entities
- **Agent traversal** — agents navigate the graph programmatically by following semantic and entity edges
- **3D visualization** (`3dGraphUniverse/`) — fly through your knowledge in 3D, billboarded labels, LOD, real-time ingest
- **MCP server** — any agent can query the graph mid-conversation via `kg_read`, `kg_query`, `kg_create`

### From NEXUS Game Engine (`programs/game_engine/`)
- **Fixed-timestep tick loop** (20 ticks/sec) — server-authoritative world state
- **Entity lifecycle** — spawn, update, destroy with proper state management
- **Flatbuffers/Protobuf schemas** — binary serialization for hot-path data (entity updates, state snapshots)
- **Spatial octree** — efficient proximity queries and interest management
- **Client-side prediction** with server reconciliation — responsive feel despite latency
- **Node-manager architecture** — each world runs as its own process, spawns/dies dynamically

### From ORACLE (`programs/oracle/`)
- **Signal ingestion → fusion → reasoning → execution pipeline** — how to process real-time data into actionable intelligence
- **PostgreSQL + Redis hybrid** — Redis for real-time pub/sub + ephemeral state, Postgres for permanent records
- **Contract-first development** — Pydantic models as the single source of truth for all data shapes
- **Self-improving knowledge base** — post-mortems feed back into the system to improve future decisions

### From CouncilAI (if visible in workspace)
- Multi-agent deliberation — multiple AI agents with different perspectives debating to reach better decisions
- Council voting / consensus mechanisms
- How agents can disagree productively and surface blind spots

---

## The Server Model — Decentralized Compute

The personal website runs on a **hybrid local-first + decentralized network** model:

### Option A: Centralized (MVP)
- User's world runs on a Hetzner VPS (like Dreamworld spec)
- Free tier: limited compute (1 vCPU, 512MB RAM, 5GB storage)
- Paid tier: scales with usage, funded by commerce flowing through the platform
- Revenue model: percentage of transactions between businesses and customers pays for server costs

### Option B: Decentralized (Target)
- Each user's machine IS a node in the network (like a game engine node)
- Your world runs on your hardware by default
- Data can pass through the network — other nodes relay, cache, and serve your world when you're offline
- Nodes that contribute compute/bandwidth earn tokens
- The network is self-funding: businesses pay tokens for storefront hosting, tokens flow to node operators who provide the compute
- **Every node is a NEXUS game engine node** — the same tick loop, the same entity system, the same spatial partitioning — but instead of just running a game, it runs a personal universe with real files, real commerce, real AI agents

### The Economics
- **Free compute allocation** — every user gets baseline free compute (enough to run their personal world, store files, interact with AI)
- **Commerce-funded scaling** — when businesses operate storefronts in the universe, the transaction fees fund the infrastructure. The more economic activity, the more compute is available to everyone.
- **Node operator incentives** — users who run nodes (keep their machine online, contribute bandwidth) earn a share of the network's transaction fees. Running a node IS playing the game — your node is your island in the constellation.

---

## What The Plan Should Contain

The agent reading this prompt should produce a full architecture plan covering:

### 1. System Architecture
- How the personal world server works (local-first, sync to network)
- How worlds connect to each other (the graph topology)
- How files are stored (local-first with network backup)
- How the economy flows (user → business → node operator → infrastructure)
- How AI agents operate within each world

### 2. Data Model
- User account + identity
- World state (entities, objects, terrain, physics)
- File storage (mapping between 3D objects and real files)
- Inventory system (items, ownership, transfer)
- Commerce (storefronts, listings, transactions, settlement)
- Knowledge graph per user (personal) + shared (network)

### 3. The Personal World Experience
- What a new user sees when they first visit roancurtis.com
- The template world (house, office, desk, computer, bookshelf)
- How files become physical objects
- How the in-game computer works (screens backed by real compute)
- How items are earned and used
- How storefronts are visited and purchases made

### 4. Network Protocol
- How nodes discover each other
- How world state syncs across the network
- How data passes through nodes (routing, caching, replication)
- How offline worlds are served by the network
- How the Packet protocol from Dreamworld maps to this system

### 5. Commerce Layer
- How businesses create storefronts (3D spaces with real product listings)
- How payments flow (crypto? fiat? tokens? hybrid?)
- How transaction fees fund infrastructure
- How node operators get paid
- How the free tier is subsidized

### 6. AI Integration
- How each user's AI agent team operates
- How the knowledge graph powers agent context
- How agents can traverse the 3D world and interact with objects
- How the CouncilAI pattern applies (multi-agent deliberation for important decisions)
- How agents improve over time (self-improving knowledge base pattern from ORACLE)

### 7. Migration Path
- Phase 1: Static personal website at roancurtis.com (portfolio, links, about)
- Phase 2: 3D world viewer (Three.js, fly through knowledge graph)
- Phase 3: Personal world with file storage and AI agent
- Phase 4: Multiplayer — connect worlds into the network
- Phase 5: Commerce — storefronts, transactions, node operator economy
- Phase 6: Full decentralization — every node is a world

### 8. What Already Exists
- The 3D knowledge graph visualizer (`3dGraphUniverse/`) — real Gemini embeddings, ingest UI, billboarded labels
- The NEXUS game engine specs — tick loop, entity system, schemas, spatial partitioning
- The ORACLE pipeline — signal processing, reasoning, execution
- The knowledge graph MCP server — agents can already query the graph
- The workspace-blueprint architecture — self-describing fractal, agent bus, convention system

---

## Constraints

- **Browser-first** — must work in Chrome/Firefox without install (Three.js / WebGPU)
- **Local-first** — personal data stays on user's device unless explicitly shared
- **Open engine** — the code is open source; the hosted network + accumulated knowledge is the product
- **Adding is free, extracting costs** — creation, sharing, learning = free. Commerce, premium compute, enterprise = paid.
- **Soul over features** — every interaction must feel intentional. Warm minimalism with depth. Nothing ships until it feels right.

---

## Output Format

Produce the plan as a structured markdown document with clear sections, decision points marked as `[DECISION: ...]`, and unknowns marked as `[UNKNOWN: ...]`. Reference specific files in this workspace where the existing implementation lives. The plan should be buildable — not a vision doc, an architecture doc that an agent could execute against.
