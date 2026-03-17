# Session Close Protocol
## The prompt Claude runs at the end of every work session

*When a task is finishing — or when told to wrap up — run this protocol exactly.*
*The output goes into `leftOffHere.md` at the workspace root.*
*Do not skip steps. Do not summarize lazily. The next session depends on this being precise.*

---

## The Prompt (Run This Internally)

```
SESSION CLOSE — fill out leftOffHere.md

Step 1: DATE AND SESSION NUMBER
  What is today's date?
  What session number is this? (check leftOffHere.md "Last Updated" line to get previous session number, increment by 1)
  What project was active? (name + path)

Step 2: WHAT WAS ACCOMPLISHED
  List every concrete artifact created or modified this session.
  Be specific: file name, what it contains, what decision it encodes.
  Do not say "worked on X" — say "wrote X which specifies Y."
  Group by type: files created, files modified, decisions made, gaps closed.

Step 3: EXACT NEXT STEP
  What is the single most important thing to do in the next session?
  It must be:
  - Specific enough that the next session can start working immediately
  - Small enough that it could be done in one session
  - Tied to the highest-priority open gap in _meta/gaps/pending.txt
  Name the exact files to create or modify.
  Name the exact source material to reference for the content.

Step 4: KEY FILES (load order)
  List the minimum set of files the next session must read, in order, to restore full context.
  Maximum 6 files. Order matters — most contextual first.
  Never list a file that doesn't exist.

Step 5: GAP TRACKER SNAPSHOT
  Copy the current contents of _meta/gaps/pending.txt into the snapshot table.
  Mark which gap is "Do this next."

Step 6: FOLDER STRUCTURE
  Write the current folder tree of the active project (depth 2 max).
  Mark each node as: specced | stub | accepted | open

Step 7: OPEN DECISIONS
  List any ADRs or decisions that are open and will affect future design choices.
  Include which phase they block and any leading candidate options.

Step 8: WRITE THE FILE
  Overwrite leftOffHere.md with the output of steps 1-7.
  Use the exact template in leftOffHere.md — update each section in place.
  Update the "Last Updated" line to today's date and session description.
```

---

## When To Run This

- Always: when the user says they're done for the session
- Always: when a major milestone is reached (e.g., all Phase 0 ADRs resolved)
- Always: when asked to "wrap up," "log where we are," "save progress," "note where we left off"
- Optional: at natural pause points mid-session if a lot of work just landed

## Trigger phrases that should fire this protocol

- "we're done for now"
- "save where we are"
- "log this"
- "leftOffHere"
- "wrap up"
- "that's enough for today"
- "pick this up later"
- "note where we left off"

---

## What Good Looks Like

A well-written `leftOffHere.md` lets the next session:
1. Read only this file and know exactly what to do
2. Start executing the next step within 2 minutes
3. Not need to re-derive anything that was already decided

A poorly written one:
- Says "worked on game_engine" instead of "wrote ADR-003 accepting semi-implicit Euler"
- Says "next: continue the PRD" instead of "next: write entity_position_update.fbs using the spec in PRD.md Part VIII section 8.3"
- Lists 10 files to read before getting started

## The document name

The file is always called `leftOffHere.md`. It lives at the workspace root. There is only ever one. It is overwritten each session, not appended. History lives in session descriptions within the file. It is not version controlled separately — the workspace's git history (if active) captures it.
