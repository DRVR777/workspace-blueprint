# CLAUDE.md — simulation

## What Is In This Directory

| Name | Purpose |
|------|---------|
| `MANIFEST.md` | Full spec: 5-stage tick pipeline, Rapier integration, physics config, determinism contract |
| `CONTEXT.md` | Build contract — inputs, process, checkpoints, outputs |
| `src/` | Phase 0 implementation (Rust crate: nexus-simulation) |
| `output/` | Phase completion reports |

## Quick Rules For This Directory

- `run_tick` is the ONLY public entry point — everything else is internal
- Determinism is non-negotiable: same inputs → bitwise identical outputs
- Bodies processed in sorted order (by object_id) at every stage — no HashMap iteration
- Player movement applies FORCE, never direct position — physics consistency comes first
- Rapier version must be pinned exactly in Cargo.toml — no semver ranges
- Phase 0 stubs: entity AI, state machines, triggers (return no-ops, don't skip the stage)

## Cross-References

- `../../shared/contracts/simulation-contract.md` — the interface this sub-program implements
- `../../shared/contracts/world-state-contract.md` — object_record, entity_record, change_request shapes
- `../../_planning/adr/ADR-003-physics-integrator.md` — semi-implicit Euler (Rapier handles internally)
- `../../_planning/adr/ADR-004-collision-detection.md` — proven library = Rapier
- `../../_planning/adr/ADR-015-technology-stack.md` — Rust + Rapier + standalone ECS
- `../spatial/MANIFEST.md` — spatial index contract
- `../node-manager/MANIFEST.md` — the caller: node-manager invokes run_tick each tick
