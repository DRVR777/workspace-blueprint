# ADR-005: GPU Texture Compression Format
Status: accepted
Date: 2026-03-14
Blocking: Phase 1

## Context
Textures are the dominant factor in asset size. Storing and transmitting uncompressed textures would make the asset pipeline impractically large. GPU-native compressed formats allow textures to be uploaded directly to GPU memory without CPU decompression — this is critical for smooth streaming.

The challenge: there is no single compressed texture format supported by all GPUs. Desktop GPUs (Windows, Linux) support BC7/DXTC. Mobile GPUs and Apple Silicon (M1/M2/M3) prefer ASTC. Some older mobile GPUs only have ETC2.

## Format Comparison

| Format | Compression ratio | Quality | Desktop | Apple Silicon | Android | Notes |
|--------|-----------------|---------|---------|--------------|---------|-------|
| BC7 | 4:1 to 8:1 | Excellent | Yes | No (fallback) | No | Best desktop quality |
| ASTC | 6:1 to 25:1 variable | Excellent | Limited | Yes (native) | Modern only | Variable block size |
| ETC2 | 4:1 | Good | No | No | Baseline | Old mobile baseline |
| BC1/DXT1 | 8:1 | Poor | Yes | No | No | Old, low quality |
| Uncompressed | 1:1 | Perfect | Yes | Yes | Yes | Too large for streaming |

## Decision

**Store each texture asset in two GPU-native formats: BC7 and ASTC.**

Workflow:
1. Source textures are stored uncompressed in the asset authoring pipeline
2. At asset publish time, the pipeline generates both BC7 and ASTC variants
3. The asset store stores both variants under the same `(type_id, lod_tier, version)` key with a format tag
4. At connection time, the client declares its GPU capability: `GPU_CAPS: bc7 | astc | both | neither`
5. The node serves the format matching the client's declaration
6. If the client declares `neither` (very old GPU), the node serves the ASTC variant and the client does CPU transcoding to RGBA — acceptable only for very old hardware, performance will be degraded

**Storage cost**: Approximately 2× per texture compared to one format. Acceptable — geometry and texture data sizes are small compared to the cost of runtime transcoding.

## Consequences

- Asset publish pipeline must include a texture compression step (BC7 encoder + ASTC encoder)
- The asset store API gains a `format` parameter: `get_asset(type_id, lod_tier, version, format)`
- Clients must send GPU capability in the HANDSHAKE message (see network protocol, Part VIII)
- The HANDSHAKE message schema gains a `gpu_caps` field (bitmask: bit 0 = BC7, bit 1 = ASTC)
- Impostor images (LOD tier 4) are stored as compressed images, not GPU-native — they are small enough that this does not matter
