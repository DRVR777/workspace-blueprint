# MANIFEST — _meta/scripts/

## Envelope
| Field | Value |
|-------|-------|
| `id` | meta-scripts |
| `type` | scripts |
| `depth` | 2 |
| `parent` | _meta/ |
| `status` | active |

## What I Am
Python automation scripts for workspace operations.
All scripts are standalone — no dependencies beyond the Python standard library.
Each script prints instructions on how to use it when run with no arguments.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| new_project.py | file | Clone `programs/_template/` to create a new project |
| run_gaps.py | file | Parse pending.txt files and surface gap candidates for agent review |
| scaffold_manifest.py | file | Generate MANIFEST.md stub for any folder; infers depth/type/parent from path |
| status.py | file | Scan all output/ folders and report workspace completion status (`status` trigger) |
| validate_manifests.py | file | Check all MANIFESTs for schema compliance; find folders missing MANIFEST.md |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| Create a new project from a PRD | new_project.py |
| Surface pending inferences for gap detection | run_gaps.py |
| New folder was just created | scaffold_manifest.py |
| User typed `status` trigger | status.py |
| Session ending — check for MANIFEST gaps | validate_manifests.py |

## Usage Quick Reference
```
python _meta/scripts/new_project.py <name> [--prd "text" | --prd @file.md]
python _meta/scripts/run_gaps.py [--all | --scope <project>]
python _meta/scripts/scaffold_manifest.py <folder-path> [--type <type>] [--update-parent]
python _meta/scripts/status.py [--project <name>] [--summary]
python _meta/scripts/validate_manifests.py [--fix] [--missing]
```
