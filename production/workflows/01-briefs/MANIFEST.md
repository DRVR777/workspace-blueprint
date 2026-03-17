# MANIFEST — 01-briefs

## Envelope
| Field | Value |
|-------|-------|
| `id` | production-workflows-01-briefs |
| `type` | stage |
| `depth` | 3 |
| `parent` | production/workflows/ |
| `status` | active |

## What I Am
Stage 1 of the production pipeline. The input stage. Contains plain-language
descriptions of what to build, handed off from writing-room/final/.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| [slug].md | file | varies | A brief — plain language, describes what to build and why |

## What I Need From Parent
- A finalized draft from writing-room/final/ copied here as [slug].md
- A valid brief contains: scope (what to build), purpose (why), audience (who),
  success criteria (how we know it's done). No technical implementation details.

## What I Give To Children
Nothing — this is a receiving stage, not a dispatching stage.

## What I Return To Parent (→ 02-specs)
A complete brief file [slug].md that spec agents can transform into a technical plan.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| You have a brief and need a technical plan | Read the brief here, then go to ../02-specs/ |
| You need to understand what a valid brief looks like | Read this MANIFEST "What I Need" section |
| You need production pipeline routing | Go to ../CONTEXT.md |

## Gap Status
No open gaps.
