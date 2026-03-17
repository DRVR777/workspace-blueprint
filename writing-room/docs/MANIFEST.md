# MANIFEST — writing-room/docs

## Envelope
| Field | Value |
|-------|-------|
| `id` | writing-room-docs |
| `type` | reference |
| `depth` | 2 |
| `parent` | writing-room/ |
| `status` | active |

## What I Am
Stable reference knowledge for writing agents. Load on demand per task type.
These files do not change during a run — they are configuration, not workflow.

## What I Contain
| File | Load When | Skip When |
|------|-----------|-----------|
| voice.md | Any writing task | Never — always load for writing |
| style-guide.md | Blog posts, tutorials, formatted docs | Quick notes, research, internal docs |
| audience.md | Tutorials, content targeting specific segments | General blog posts, internal docs |

## Shared Dependency Warning
voice.md is also consumed by community/ cross-workspace.
Changes to voice.md affect community output. See gap-002 in _meta/gaps/.

## What I Need From Parent
Nothing — these are terminal reference files. Load them from the parent workspace.

## What I Give To Children
Nothing — these are read-only reference files, not dispatching folders.

## What I Return To Parent
Voice, style, and audience specifications that constrain writing agent output.

## Routing Rules
Never load all three files simultaneously unless writing a tutorial for a specific audience.
For blog posts: voice.md + style-guide.md.
For tutorials: voice.md + audience.md + style-guide.md.
For edits only: voice.md + the draft.
For research only: load nothing from this folder.

## Gap Status
| Gap ID | Description |
|--------|-------------|
| gap-002 | voice.md is a cross-workspace shared resource but writing-room doesn't document this (open — see _meta/gaps/) |
