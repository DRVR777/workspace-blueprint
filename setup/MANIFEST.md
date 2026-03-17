# MANIFEST — setup/

## Envelope
| Field | Value |
|-------|-------|
| `id` | root-setup |
| `type` | onboarding |
| `depth` | 1 |
| `parent` | workspace root |
| `status` | active |

## What I Am
Onboarding layer. Runs when a human types `setup`. Extracts the domain-specific rules,
voice, and constraints for a new workspace domain via a two-pass questionnaire.
Output hydrates `brand-vault/` and generates voice rules for `shared/voice.md`.

## What I Contain
| Name | Type | Purpose |
|------|------|---------|
| questionnaire.md | file | Two-pass onboarding: quick answers → agent drafts rules → human edits |

## Routing Rules
| Condition | Go To |
|-----------|-------|
| User types `setup` | questionnaire.md |
| Review generated voice rules | questionnaire.md §Output |
