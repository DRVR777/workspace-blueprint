# Voice Rules — {{PROJECT_NAME}}

<!--
HOW TO USE: Replace placeholders below during project onboarding (run `setup` trigger).
These rules are enforced as error conditions, not style preferences.
Reference: _core/CONVENTIONS.md §12 (P-12)
-->

## Hard Constraints

<!-- Numbered, binary. Each is a prohibited pattern with a reason. -->

ERROR-01: [prohibited pattern] — [why this breaks the voice]
ERROR-02: [prohibited pattern] — [why this breaks the voice]
ERROR-03: [prohibited pattern] — [why this breaks the voice]

## Sentence Rules

<!-- Do/don't pairs with verbatim example sentences extracted from real content. -->

DO: "[verbatim example of correct voice]"
DON'T: "[verbatim example of wrong voice]"
WHY: [specific reason — not vague like "sounds better"]

DO: "[verbatim example of correct voice]"
DON'T: "[verbatim example of wrong voice]"
WHY: [specific reason]

## Pacing

<!-- Rhythm notation. Describe the pattern, then what to avoid. -->

Pattern: [Long sentence. Short sentence. Fragment. Then payoff.]
Never: [three long sentences in a row / two short sentences side by side / etc.]

## Audit Checklist

Run before committing any content output:

- [ ] No ERROR-01 violations present
- [ ] No ERROR-02 violations present
- [ ] No ERROR-03 violations present
- [ ] Every sentence pair follows the Sentence Rules
- [ ] Pacing rhythm matches the Pattern above

## How These Rules Were Derived

Source: questionnaire.md two-pass process — concrete example sentences, not adjectives.
If rules need to be updated: edit this file, re-run setup, log the change to _meta/gaps/pending.txt.
