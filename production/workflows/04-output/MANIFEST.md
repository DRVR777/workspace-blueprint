# MANIFEST — 04-output

## Envelope
| Field | Value |
|-------|-------|
| `id` | production-workflows-04-output |
| `type` | stage |
| `depth` | 3 |
| `parent` | production/workflows/ |
| `status` | active |

## What I Am
Stage 4 of the production pipeline. The deliverable stage. Nothing lands here
without passing the spec's acceptance criteria. Versioned, tested, final.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| [slug]-v[n].[ext] | file | varies | Finished deliverable. v1 = first complete. v2 = after first feedback round. |

## What I Need From Parent (← 03-builds)
- Completed, tested build from ../03-builds/[slug]/
- Original spec from ../02-specs/[slug]-spec.md (used as acceptance criteria)
- /webapp-testing verification passed (or manual verification documented)

## What I Give To Children
Nothing — this is the terminal stage.

## What I Return To Parent
- Final deliverable file [slug]-v[n].[ext]
- This output is consumed by community/ — see community/CONTEXT.md for how

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Content needs repurposing for social/newsletter/events | community/ |
| Build failed acceptance criteria | Return to ../03-builds/ |
| You need to understand the pipeline | Go to ../CONTEXT.md |

## Gap Status
| Gap ID | Description |
|--------|-------------|
| gap-001 | This stage's output is consumed by community/ but production/CONTEXT.md does not document this (open — see _meta/gaps/) |
