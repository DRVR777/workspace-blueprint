# MANIFEST — _meta/guards/

## Envelope
| Field | Value |
|-------|-------|
| `id` | meta-guards |
| `type` | meta |
| `depth` | 2 |
| `parent` | _meta/ |
| `status` | active |

## What I Am
Protective guards that run during or after sessions. Guards prevent state corruption —
they catch problems that slip past the fix-first rule or the normal gap system.

Two guards currently implemented:
- **Guard 3** (ALWS §15): finalized.flag cascade — blocks writes to finalized artifacts
- **Guard 5** (ALWS §6): MANIFEST reconciliation — detects MANIFEST staleness

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| guard-03-finalized-flag.md | file | Prevents writes to any folder containing finalized.flag |
| guard-05-manifest-reconcile.md | file | Detects MANIFEST files whose What I Contain is stale |

## What I Need From Parent
- Read access to all folders (guard-03 scans for finalized.flag, guard-05 scans MANIFEST files)

## What I Give To Children
Nothing. No children.

## What I Return To Parent
- Blocked write attempts (guard-03) → logged to `_meta/gaps/pending.txt` as `critical`
- Stale MANIFEST list (guard-05) → logged to `_meta/gaps/pending.txt` as `high`

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Check before writing to any artifact folder | guard-03-finalized-flag.md |
| Check MANIFEST freshness across workspace | guard-05-manifest-reconcile.md |

## Layer 0 Test
Guards check file system state. They have no knowledge of what files contain.
