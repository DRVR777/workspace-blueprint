# MANIFEST — 02-specs

## Envelope
| Field | Value |
|-------|-------|
| `id` | production-workflows-02-specs |
| `type` | stage |
| `depth` | 3 |
| `parent` | production/workflows/ |
| `status` | active |

## What I Am
Stage 2 of the production pipeline. The planning stage. Contains technical
specifications (contracts) that define WHAT to build and acceptance criteria.
NOT how to build it — implementation is Stage 3's creative domain.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| [slug]-spec.md | file | varies | A spec contract — scope, acceptance criteria, tech approach (high-level), dependencies |

## What I Need From Parent (← 01-briefs)
- Brief file [slug].md from 01-briefs/
- tech-standards.md from ../../docs/ (load this when writing specs)
- Context7 MCP (fetch current library docs) and Web Search MCP (research) available at this stage

## What I Give To Children (→ 03-builds)
- A spec contract [slug]-spec.md that defines WHAT and acceptance criteria
- Builder has full creative freedom for HOW, constrained only by tech-standards.md

## What I Return To Parent
A complete spec [slug]-spec.md that build agents can implement.

## Routing Rules
| Condition | Go To |
|-----------|-------|
| You have a spec and are ready to build | Go to ../03-builds/ |
| You need to understand the pipeline | Go to ../CONTEXT.md |
| You need library docs for this spec | Use Context7 MCP |

## Gap Status
No open gaps.
