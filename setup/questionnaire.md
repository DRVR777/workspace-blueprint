# SETUP — Workspace Onboarding Questionnaire

Triggered by: user types `setup`

This is a two-pass process. Pass 1 collects raw answers. Pass 2 refines them into
machine-checkable rules. The human edits Pass 2 output before rules become binding.

Do not skip passes. Do not invert order. Do not summarize answers — use exact quotes.

---

## Pass 1 — Quick Answers

Ask each question. Record the human's exact words — do not paraphrase.
Ask all questions before moving to Pass 2.

---

### Q1 — Domain
What is this workspace for? (One sentence: what kind of work, for what audience.)

*Record verbatim.*

---

### Q2 — Programs
What are the main components or deliverables? List each one.

*Record as a list — one item per line.*

---

### Q3 — Voice — Right Example
Give me one sentence that sounds exactly right for this workspace. The most "us" sentence you can write.

*Record verbatim. This becomes a DO example in voice rules.*

---

### Q4 — Voice — Wrong Example
Give me one sentence that sounds completely wrong. The most "not us" sentence.

*Record verbatim. This becomes a DON'T example in voice rules.*

---

### Q5 — Hard Constraints
What is the one thing content from this workspace must never do or say?
(Can be a tone, a claim, a format, a word.)

*Record verbatim. Becomes ERROR-01 in Hard Constraints.*

---

### Q6 — Pacing
Read this out loud and tell me which sounds like your workspace:

Option A: "Short sentences. Direct. One idea at a time."
Option B: "Longer sentences that build context before landing on the point."
Option C: "A mix — short when punchy, long when complexity demands it."

*Record their choice and any additional pacing notes.*

---

### Q7 — What Success Looks Like
Complete this sentence: "We'll know this workspace worked when ___."

*Record verbatim.*

---

## Pass 2 — Agent Drafts Rules

After Pass 1 is complete, generate draft rules using the verbatim answers.
Present the drafts to the human. They edit. Only after editing do these become binding.

---

### Draft: Hard Constraints

Using Q5 and Q4 answers:

```
## Hard Constraints

ERROR-01: [Q5 answer rephrased as a prohibited pattern] — [why, from Q4's wrong example]
```

Add additional ERROR entries if Q3/Q4 reveal more binary violations.

---

### Draft: Sentence Rules

Using Q3 and Q4 answers:

```
## Sentence Rules

DO: "[Q3 verbatim]"
DON'T: "[Q4 verbatim]"
WHY: [one sentence: what makes the DO example work]
```

Add additional DO/DON'T pairs if the answers reveal more contrasts.

---

### Draft: Pacing

Using Q6 answer:

```
## Pacing

[Derive rhythm notation from their Q6 choice]

Option A selected → "Short sentence. Then shorter. Fragment. Payoff."
Option B selected → "Build the context (one clause, then another), then land the point at the end."
Option C selected → "Short sentence. Short. Then a longer one when the idea needs room to breathe. Then short again."
```

---

### Draft: Value Framework Check

Using Q7 answer — identify which 2+ value criteria (NOVEL, USABLE, QUESTION-GENERATING, INTERESTING)
the workspace is optimized for:

```
## Primary Value Criteria

This workspace primarily targets: [list 2+ from Q7 mapping]

Mapping rationale: [one sentence per criterion selected]
```

---

## Output

After human edits Pass 2 drafts:

1. Write voice rules to `programs/[PROJECT_NAME]/shared/voice.md`:
   ```markdown
   # Voice Rules — [PROJECT_NAME]

   ## Hard Constraints
   [Final ERROR entries]

   ## Sentence Rules
   [Final DO/DON'T pairs]

   ## Pacing
   [Final rhythm notation]

   ## Value Framework
   [Primary criteria + rationale]
   ```

2. Write domain summary to `programs/[PROJECT_NAME]/_planning/domain.md`:
   ```markdown
   # Domain — [PROJECT_NAME]

   ## What This Workspace Is For
   [Q1 verbatim]

   ## Programs / Deliverables
   [Q2 list]

   ## Success Criterion
   [Q7 verbatim]
   ```

3. Log to root `_meta/gaps/pending.txt`:
   ```
   [timestamp] | root | setup/questionnaire.md | inferred "setup complete for [PROJECT_NAME] — voice rules written, domain documented" — setup trigger
   ```

4. Report to human:
   ```
   SETUP COMPLETE
   ==============
   Voice rules written to: programs/[PROJECT_NAME]/shared/voice.md
   Domain summary written to: programs/[PROJECT_NAME]/_planning/domain.md

   Next: type `intake: "[prd text]"` to scaffold programs, or open _planning/CONTEXT.md to plan.
   ```

---

## One Rule

All voice rules must come from the human's exact words.
No agent-invented style guidance. No adjectives that weren't spoken.
If Pass 1 produced no concrete examples — ask again before drafting.
