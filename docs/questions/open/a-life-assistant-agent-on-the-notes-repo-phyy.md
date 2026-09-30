---
id: phyy
title: A life assistant: an agent that runs the human's notes repo through the meta-notes CLI
opened: 2026-09-30
repos: [bridle, meta-notes, notes]
changes: []
specs: []
needs: []
see: [rs7p, u6wk, k4wq, ajqa]
---

## The ask

The human, verbatim (2026-09-30):

> New or updated project: similar to bridle an agent that knows house to use my personal notes
> repo using the meta notes CLI to help manage my life. Web chat or telegram eventually start
> with remote control for note.
>
> I want to send meters to create tasks and reminders and record events and things that I want
> to renderer. And check on projects and things like maintenance tasks

The advisor's reading of the dictation (to confirm): "knows how to use", "for now", "send
messages", "things that I want to remember".

## What it is

A standing agent, like bridle's orchestrator, whose job is the human's life admin rather than
code. The human talks to it; it reads and writes the notes repo only through the meta-notes CLI.

- **Capture:** from a message, create a task, a reminder, or a recorded event, or file something
  to remember, in the right place.
- **Check in:** on projects (`project/`) and recurring upkeep such as maintenance tasks
  (`area/`): what's due, what's stale, what's next.
- **Talk to it:** Remote Control first. Web chat or Telegram later. Email from rs7p fits too
  (e.g. `notes@dev.branam.us`).

## What exists (checked 2026-09-30)

- **The notes repo:** `/Volumes/Data/files/notes` (`github.com/jonathanbranam/notes`), PARA layout
  (`project/`, `area/`, `resource/`, `archive/`) with a `CLAUDE.md` describing it. `project/life-app`
  (Proposal, Time Blocking, Time Tracking) looks like earlier thinking on the same idea. Not a
  bridle project.
- **meta-notes:** the CLI is `bin/meta-notes` in `/Volumes/Data/work/meta-notes-workspace/meta-notes`,
  being onboarded to bridle on `bridle-adopt` (ajqa).
- **The standing-session pieces are in bridle already:** an external session in a tagged tmux pane,
  kept running and restarted at context limits by the daemon (fx7x), and a wake command. Today
  they're built for the orchestrator only.

## Open questions for the human

1. **New project or part of meta-notes?** A separate project (its own repo holding the role
   prompt and rules, serving the notes repo), or a role that ships with meta-notes? The advisor
   leans to a role and rules kept in bridle's `workflow/` (a "life" pack) plus a small project
   entry for the notes repo, so no new code repo.
2. **Does the notes repo become a bridle project?** Needed for the daemon to supervise the agent
   in it. By the existing-projects rule, nothing in the notes repo changes without the human's
   review; its `CLAUDE.md` stays theirs.
3. **What the meta-notes CLI lacks** for capture (reminders, events, recurring maintenance). A
   short survey of its commands against the verbs above tells us what meta-notes must add.
4. **Reminders need a clock:** something has to tell the human when one is due. The rs7p digest,
   Remote Control, or both?
5. **Which machine:** the NUC (runs meta-notes, always on) seems the natural home.
6. **Git:** does the agent commit and push the notes repo itself, and on which branch?
