# Style Guide

<!--
TEACHING NOTE: Separate from voice. Voice = personality.
Style guide = formatting rules. An agent can load one without the other.
-->

## Structure

- **Blog posts:** 800-1500 words. One main idea. Hook in the first sentence.
- **Tutorials:** Step-by-step. Numbered. Each step starts with what the reader does, then why.
- **Technical docs:** Reference-style. Scannable. Tables over paragraphs when comparing things.

## Formatting

- Headers: H2 for sections, H3 for subsections. Never H1 (that's the title).
- Code blocks: Always specify the language. Always show realistic examples, not `foo`/`bar`.
- Links: Descriptive text, never "click here."
- Lists: Use bullets for unordered items, numbers for sequential steps.

## Length

Let the idea dictate length. A 400-word post that nails one insight beats a 2000-word post that meanders. But don't artificially compress — if the topic needs depth, give it depth.

---

## Content Frame Patterns

The TYPE (blog post, tutorial, technical doc) sets the container. The FRAME sets the internal structure. When you know the frame, the structure follows directly — no inference needed.

| Content Frame | Structure | Length | Opens With |
|---------------|-----------|--------|------------|
| **Best practices** | Numbered list. Each item: problem it prevents → the practice → one concrete example. No filler between items. | 1000–1500 words | The consequence of skipping this: "Auth is the first thing that breaks at scale." |
| **Comparison / tradeoff** | Context (when does this choice arise) → options table (2–4 options × 3–5 criteria) → recommendation with named conditions. | 800–1200 words | The decision scenario: "You're choosing between X and Y. Here's the decision." |
| **Conceptual explainer** | Analogy → mechanism (how it actually works) → worked example → one implication the reader probably didn't see coming. | 600–1000 words | The problem the concept solves, not the concept itself: "X exists because Y kept failing." |
| **Launch / announcement** | News first → what specifically changed → who it helps and how → one sentence on how to start. No preamble, no history. | 400–700 words | The specific change, stated flatly: "As of today, [product] does [thing]." |
| **Troubleshooting** | The symptom (exact error or behavior) → root cause → fix (numbered steps) → how to prevent recurrence. | 500–900 words | The error message or observable symptom, verbatim or described exactly. |
| **Case study** | Problem state (before) → what was tried and failed → the solution → measurable outcome. Evidence at every stage. | 1000–2000 words | The problem as it existed before the solution: "Before [thing], [team] spent [cost] on [problem]." |
