# _planning — workspace-builder Architecture

## What Lives Here

| File | Purpose |
|------|---------|
| `prd-source.md` | Source PRDs: MWP v1, MWP v2, ALWS — the authoritative requirements |
| `roadmap.md` | All PRD requirements tracked with implementation status |
| `adr/` | Architecture decisions for the builder itself |

## Task Routing

| Your Task | Do This |
|-----------|---------|
| See what PRDs require | Read `prd-source.md` |
| Check implementation status | Read `roadmap.md` |
| Update implementation status | Edit `roadmap.md` — change ❌ to ⚠️ to ✅ |
| Extract new requirements after PRD update | Run `../programs/prd-tracker/CONTEXT.md` |

## Hard Rule
`roadmap.md` is the single source of truth for what's built vs not built.
Never duplicate this information elsewhere. If a requirement is implemented,
mark it ✅ in the roadmap. Nowhere else.
