# MANIFEST — _archive/

## Envelope

| Field | Value |
|-------|-------|
| `id` | workspace-archive |
| `type` | archive |
| `depth` | 1 |
| `parent` | workspace root |
| `version` | 1.0.0 |
| `status` | active |

## What I Am
Long-term archive for content that is no longer active in the workspace. Contains deprecated projects, session history files, and any content that has completed the deprecation protocol (see `_meta/deprecation-protocol.md` when created).

## What I Contain

| Name | Type | Status | Purpose |
|------|------|--------|---------|
| session-histories/ | folder | active | Raw Claude conversation history files — moved here from workspace root |

## Routing Rules

| Condition | Go To |
|-----------|-------|
| Looking for a deprecated project | Browse this folder directly |
| Looking for an old session history | session-histories/ |
| Deciding if something should be archived | See deprecation-protocol.md in _meta/ |

## Archive Rules

1. Nothing is deleted from _archive/ — only added.
2. Content may be read but never referenced by active workspace files.
3. After 90 days in _archive/ with no reads, content may be permanently deleted with a log entry.
