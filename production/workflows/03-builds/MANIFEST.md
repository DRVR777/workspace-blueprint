# MANIFEST — 03-builds

## Envelope
| Field | Value |
|-------|-------|
| `id` | production-workflows-03-builds |
| `type` | stage |
| `depth` | 3 |
| `parent` | production/workflows/ |
| `status` | active |

## What I Am
Stage 3 of the production pipeline. The execution stage. Where code gets written,
demos get assembled, and things get built. Builders have full creative freedom
within the quality floor set by the spec and the reference docs.

## What I Contain
| Name | Type | Status | Purpose |
|------|------|--------|---------|
| [slug]/ | folder | varies | One sub-folder per build — all build files for that project |

## What I Need From Parent (← 02-specs)
- Spec contract [slug]-spec.md from ../02-specs/
- design-system.md from ../../docs/ (visual standards, color tokens, typography)
- component-library.md from ../../docs/ (available components, packages)
- tech-standards.md from ../../docs/ (code quality, testing requirements)

## Skills Active At This Stage
- /frontend-design — for web-based deliverables
- /webapp-testing — verify builds work (Playwright-based browser testing)
- Context7 MCP — look up API details mid-build if needed

## What I Give To Children
- The spec (as acceptance criteria)
- Reference docs loaded from ../../docs/
- Full creative freedom for implementation details

## What I Return To Parent (→ 04-output)
- Working, tested build in [slug]/ sub-folder
- Build must pass acceptance criteria from the spec before moving to output

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Build is complete and tested | Copy deliverable to ../04-output/ |
| You need component reference | Read ../../docs/component-library.md |
| You need design reference | Read ../../docs/design-system.md |
| You need to understand what "done" means | Read the spec from ../02-specs/ |

## Gap Status
No open gaps.
