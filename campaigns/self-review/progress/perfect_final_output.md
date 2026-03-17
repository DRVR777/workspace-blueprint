# Perfect Final Output — self-review campaign

*This document describes the ideal end state of this campaign. No code. No filenames. Pure outcome description. Extremely detailed.*

---

## What Success Looks Like

When this campaign is complete, the workspace-blueprint is a system that has internalized its own critique and resolved every structural contradiction it contained. Not patched — resolved. Every fix becomes a pattern. Every gap becomes a closed loop. Every contradiction becomes a new rule that prevents the same contradiction from arising again.

The workspace does not merely function. It is structurally honest about what it is, what it can do, and — most importantly — what it cannot do. Systems that pretend to have capabilities they don't have are dangerous. Systems that openly document their limitations are trustworthy.

---

## The Four Outcomes This Campaign Produces

### Outcome 1: The Workspace Is Self-Consistent

Every file that claims something is true agrees with every other file that claims the same thing. Status fields match across REGISTRY, MANIFEST, and leftOffHere. Path references are relative, not absolute, so the workspace moves without breaking. Template folders contain exactly what every cloned project gets. No file references a pattern that has silently changed meaning. The archive contains everything that should be archived and the root contains only what should be active.

A person or agent who reads any file in the workspace and then reads any other file will not encounter a contradiction. They may encounter incomplete information, but incomplete information is documented as incomplete. Contradictions are the failure mode this campaign eliminates.

### Outcome 2: The Workspace's Rules Are Honest About Their Limits

The Terry Davis Constraint is the most important principle in the workspace. But it has an honest limitation: it requires human judgment and cannot be run mechanically. The perfect outcome is not to make it mechanical — it cannot be made mechanical. The perfect outcome is to say so, clearly, in the file that defines the constraint. Then the constraint is real because it is honest about what it is.

Similarly, the Fix-First Rule is powerful but it can eat entire sessions if run without scope boundaries. The perfect outcome adds a scope boundary to the rule — not to weaken it, but to make it actually executable in bounded time.

Every rule in CONVENTIONS.md has been examined for hidden assumptions and unstated limitations. Every hidden assumption is now stated. Every unstated limitation is now documented. The rules are not weakened — they are made trustworthy by being honest.

### Outcome 3: The Workspace's Gaps Are Closed Loops, Not Lists

Every bad thing identified in this campaign has a corresponding gap object in the gap registry. Every gap object has a type, a severity, a description of what closed means, and an agent-executable prompt to close it. The gaps are not a list of "things to fix someday." They are an ordered, prioritized, prompt-ready work queue.

The next agent that enters this workspace for a maintenance session can read the gap registry, pick the highest-severity open gap, follow the execution prompt, and close it. No re-analysis required. No human needed for any gap with `requires_human: false`. The workspace generates its own next tasks — which is what it was designed to do.

### Outcome 4: The Products Get Built

This campaign's most uncomfortable finding is BAD-01: the infrastructure-to-product ratio is inverted. The meta-system is mature. The products are specced. Nothing is built.

The perfect output of this campaign includes a concrete resolution to this: ORACLE Phase 1 signal-ingestion begins in the next build session. NEXUS Phase 0 node-manager begins in the session after. The meta-system is frozen at its current state — no further improvements, no new patterns, no new campaigns about the workspace itself — until at least one product has working code in production.

The workspace's purpose is to produce products. The meta-system's purpose is to support product-building. When the meta-system becomes the product, the workspace has failed its own purpose.

---

## The Shape of the Workspace After This Campaign

**The root is clean.** No Windows artifacts. No unmanaged files. No raw conversation history. Every file at the root has a MANIFEST entry. Every folder at the root has a MANIFEST.md.

**The patterns are tiered.** CONVENTIONS.md has four tiers: foundation, enforcement, quality, refinement. An agent entering for the first time reads the five foundation patterns and can operate. It reads the full 28 only when needed.

**The rules have scope.** Fix-First has a scope boundary. ADR blocking has a threshold. The Terry Davis test has a documented limitation. No rule tries to do more than it can do.

**The sessions have integrity.** leftOffHere.md uses workspace-relative paths. Multi-agent sessions use the snapshot-and-merge protocol. Mid-session checkpoints prevent state loss. The session ends with a REGISTRY sync, a MANIFEST completeness check, and a routing validation.

**The products are moving.** The leftOffHere.md top section reads: "BUILD SESSION — oracle signal-ingestion in progress." Meta-system work is queued, not blocked, but clearly secondary to shipping code.

**The template is complete.** `programs/_template/` includes a campaigns/ folder. Every project cloned from it gets the campaign pattern automatically. No project has to discover it ad-hoc.

**The archive exists.** `_archive/` holds deprecated content and session histories. Nothing is deleted without first being archived. Nothing sits at the root past its useful life.

---

## The Metric for Done

This campaign is complete when:

1. All 25 action items in `clarification.md §4` are marked complete.
2. The workspace root contains no unmanaged files.
3. CONVENTIONS.md has Pattern Tiers, version numbers on all 28 patterns, and a CONVENTIONS-changelog.md.
4. REGISTRY.md status fields match all project MANIFEST.md status fields.
5. leftOffHere.md contains no absolute paths.
6. programs/_template/ contains a campaigns/ folder.
7. _archive/ exists and contains claudehist + deprecated root projects.
8. At least one of the following is true: oracle has production Python code, or NEXUS has a running node-manager.

Items 1–7 are agent-executable. Item 8 requires actual product building sessions.

---

*This document is the north star. All session work tracks against these four outcomes and eight completion criteria.*
