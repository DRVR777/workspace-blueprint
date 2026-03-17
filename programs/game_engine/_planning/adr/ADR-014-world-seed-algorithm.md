# ADR-014: World Seed / Procedural Generation Algorithm
Status: accepted
Date: 2026-03-14
Blocking: Phase 0

## Context
The world generator (PRD Part III.4) produces terrain deterministically from a position + seed. The algorithm determines:
- Visual quality of terrain (is it interesting? Does it look natural?)
- Computational cost (terrain is generated on-demand, must be fast)
- Reproducibility (same position + seed must always yield identical output)
- Scalability (must work over effectively infinite coordinate ranges without repeating or artifacts)

The generator outputs: heightmaps, biome assignments, and feature placer distributions. Each output layer uses a different noise function or derivation.

## Options Considered

**Classic Perlin Noise**
- The historical standard
- Has visible grid artifacts at certain frequencies
- Square grid alignment shows in terrain (45-degree ridges appear)
- Verdict: Outdated — superseded by Simplex

**Simplex Noise (Ken Perlin's 2001 patent-expired version)**
- Fewer artifacts than Perlin, works in N dimensions efficiently
- Triangular grid avoids the 45-degree artifact
- O(N²) in N dimensions vs O(N·2^N) for Perlin — faster in 3D and above
- Patent originally held concerns — now expired/clarified
- Verdict: Strong baseline

**OpenSimplex2 / FastNoiseLite**
- Open-source implementations of simplex-family noise without patent concerns
- Available as a single-file library in multiple languages
- FastNoiseLite specifically: one header/file, supports multiple noise types, GPU-friendly
- Verdict: Best practical choice — free, battle-tested, portable

**Domain-Warped Layered Noise**
- Instead of sampling noise at (x, z), sample at (x + noise(x,z), z + noise(x+offset, z+offset))
- Domain warping produces organic, non-grid-aligned terrain features (cliffs, river valleys, eroded shapes)
- Combined with multiple octaves (fractal noise) — each octave adds finer detail
- Cost: 2-4 noise samples per output value instead of 1
- Verdict: The industry standard for high-quality procedural terrain (Minecraft, Valheim, etc. all use this)

## Decision

**Domain-warped fractal Simplex noise using FastNoiseLite (or equivalent single-file library).**

Specifically, the terrain height at (x, z) is computed as:

```
ABSTRACT SPECIFICATION (not code):

  WARP LAYER:
    warp_x = SIMPLEX_NOISE(x * WARP_FREQUENCY, z * WARP_FREQUENCY, seed + WARP_OFFSET_X)
    warp_z = SIMPLEX_NOISE(x * WARP_FREQUENCY, z * WARP_FREQUENCY, seed + WARP_OFFSET_Z)

  HEIGHT LAYER (fractal, 5 octaves):
    height = 0
    amplitude = 1.0
    frequency = BASE_FREQUENCY
    FOR octave = 0 to 4:
      height += SIMPLEX_NOISE(
        (x + warp_x) * frequency,
        (z + warp_z) * frequency,
        seed + octave * OCTAVE_SEED_OFFSET
      ) * amplitude
      amplitude *= PERSISTENCE      (0.5 — each octave is half the strength)
      frequency *= LACUNARITY        (2.0 — each octave is twice the frequency)

  FINAL_HEIGHT = height * HEIGHT_SCALE + SEA_LEVEL_OFFSET
```

Biome assignment uses separate noise channels for temperature and humidity (no domain warp — biomes should transition smoothly, not dramatically).

Feature placement uses a different technique: Poisson disk sampling driven by a noise-based density field. This gives natural-looking irregular feature distributions.

## Constants (Tuning Parameters, Not Hardcoded)

All of the following are stored in world generator configuration, not hardcoded:
- `WARP_FREQUENCY`, `WARP_OFFSET_X`, `WARP_OFFSET_Z`
- `BASE_FREQUENCY`, `PERSISTENCE`, `LACUNARITY`
- `HEIGHT_SCALE`, `SEA_LEVEL_OFFSET`
- Number of octaves

## Consequences

- The world generator sub-program takes a dependency on a noise library
- All terrain generation is reproducible: same (position, seed, config) → identical output
- Configuration parameters are part of the world record in the world graph (so the world always generates consistently regardless of config changes over time)
- The world seed is stored in the world graph at the root level — it is set once on world creation and never changes
- ADR-009 (world graph replication) decision affects how the seed is distributed to nodes — but the seed is read-only after creation, so consistency is trivially maintained
