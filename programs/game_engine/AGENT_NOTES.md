# Agent Notes — Knowledge Transfer

*Things the next agent needs to know that aren't in any spec or contract.*

---

## Build Environment Landmines

### Windows linker situation
`C:\Program Files\Git\usr\bin\link.exe` is **not** the GNU coreutils `link` command. It's `rust-lld` (LLVM linker) that was copied there to replace the original. This is a hack because:
- The original Git Bash `link.exe` (GNU coreutils) shadows MSVC's `link.exe`
- VS Build Tools are installed but the user's disk was nearly full when installing
- If Git for Windows updates, it will restore the original `link.exe` and builds will break
- Fix: re-copy rust-lld, or install VS Build Tools properly with more disk space

### Target directory
`.cargo/config.toml` sets `target-dir = "C:\\nexus-build\\target"` because the user's home directory path (`C:\Users\Quandale Dingle`) contains a space that breaks some linkers. Do not remove this setting.

### Disk space
The user's C: drive has ~20 GB free on a 932 GB drive (99% recordings). Don't install large packages without checking. The GNU Rust toolchain (`stable-x86_64-pc-windows-gnu`) could never install due to antivirus blocking `.exe` renames in `~/.rustup/`.

### Rapier version
Pinned to `rapier3d = 0.22` in Cargo.toml. This is old (current is 0.32+). Several API differences:
- `IntegrationParameters` fields changed — we use `..Default::default()` instead of listing all fields
- `ContactPair.has_any_active_contact` is a field, not a method (no parentheses)
- If upgrading, expect more API breakage — check rapier3d changelog

---

## Architecture Decisions Not Obvious From Code

### Rapier world is rebuilt every tick
`physics.rs` creates a new `RigidBodySet`, `ColliderSet`, and `PhysicsPipeline` every tick. This means:
- No warm-starting of constraint solver (slightly less stable contacts)
- No persistent Rapier state (simpler, no state management bugs)
- Performance cost: ~2ms to rebuild at 500 bodies
- Phase 1 should persist the Rapier world across ticks for better perf

### Client sends movement direction, not position
The client sends a normalized direction vector via `PLAYER_ACTION`. The server applies this as force to the player's physics body. The server is authoritative — the client never directly sets position. Client-side prediction moves the camera locally, but the EntityField renders server-authoritative positions for all entities (including the local player's capsule, which will visually lag behind the camera).

### Simulation has two entry points
- `run_tick()` — pure function, clones bodies, returns diffs. Use for replay/testing.
- `run_tick_mut()` — mutates snapshot.bodies in place. Used by the tick loop.
Both exist intentionally. The pure version satisfies the simulation contract's determinism requirement.

### ELEV8-source and personalWebsite are git submodules
They were cloned into `programs/` and committed as submodules (embedded repos). `git clone` without `--recurse-submodules` won't pull their contents. The CONTEXT.md files in each explain what's reusable.

---

## Known Issues (Not Bugs, Just Phase 0 Limitations)

1. **No auth** — HANDSHAKE payload is ignored. Anyone can connect. Phase 1 adds JWT.

2. **No reconciliation** — client moves camera immediately (prediction), server moves entity via physics. The player's capsule entity in EntityField will visually trail behind the camera. local-simulation/ spec covers the fix but isn't implemented yet.

3. **Ground plane mismatch** — Server ground is at `(500, -0.5, 500)` covering domain center. Client Terrain is at origin. They won't align until the client reads world state from server and positions terrain accordingly.

4. **All entities rendered same color** — EntityField uses PLAYER_COLOR for local player and ENTITY_COLOR for others, but doesn't know which entity is "local" from the network state until HANDSHAKE_RESPONSE is processed.

5. **No object persistence** — entities exist only in RAM. Server restart = everything despawns. Phase 1 adds PostgreSQL.

---

## File Paths That Matter

| What | Where |
|------|-------|
| Rust workspace root | `programs/game_engine/world/Cargo.toml` |
| Server entry point | `world/crates/nexus-node/src/main.rs` |
| Physics pipeline | `world/crates/nexus-simulation/src/physics.rs` |
| Wire protocol | `world/crates/nexus-node/src/protocol.rs` |
| Client WebSocket | `engine/programs/renderer/src/network/useNetworkState.ts` |
| Character controller | `engine/programs/renderer/src/components/PlayerController.tsx` |
| World stub (offline mode) | `engine/programs/renderer/src/simulation/worldStateStub.ts` |
| Hook that switches stub/network | `engine/programs/renderer/src/hooks/useWorldState.ts` |
| Building system spec | `engine/programs/building/MANIFEST.md` |
| World graph contract | `shared/contracts/world-graph-contract.md` |
| VPS deployment guide | `programs/game_engine/VPS_DEPLOY.md` |
| Stair system data structure | `engine/programs/renderer/src/components/PlayerController.tsx` (createStaircase, registerStaircase) |
| ELEV8 original stair code | `programs/ELEV8-source/lib/staircases.ts` |
| ELEV8 character controller | `programs/ELEV8-source/components/scene/AvatarController.tsx` |

---

## What's Actually Ready vs What Looks Ready

| Component | Looks like | Actually is |
|-----------|-----------|-------------|
| `nexus-core` | Library crate | **Done** — all types, math, config, constants. 15 tests pass. |
| `nexus-spatial` | Library crate | **Done** — octree with full CRUD + queries. 6 tests pass. |
| `nexus-simulation` | Library crate | **Functional** — Rapier physics works but rebuilds world each tick. |
| `nexus-node` | Server binary | **Functional** — handles connections, runs tick loop, broadcasts. Not tested with real client yet. |
| Client renderer | R3F app | **Renders** at 60+ FPS in stub mode. Network mode written but untested against server. |
| PlayerController | Movement component | **Written** with stair system. Untested in browser (no `npm install` has been run). |
| Building system | Spec only | **Spec only** — 683 lines of MANIFEST.md, zero implementation code. |
| World graph | Contract only | **Contract only** — 350+ lines defining portals, subworlds, constellation. Zero implementation. |

---

## The Next Session Should

1. Deploy server to VPS (`VPS_DEPLOY.md`)
2. Run `npm install && npm run dev` in `engine/programs/renderer/`
3. Set `VITE_NEXUS_SERVER=ws://VPS_IP:9001`
4. Open browser → click to lock cursor → press WASD
5. Open second tab → see both players
6. Fix whatever breaks (it will be the first real integration test)
