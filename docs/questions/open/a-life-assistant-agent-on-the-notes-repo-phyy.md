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

## The human's answers (2026-09-30)

The human, verbatim:

> Hmm yeah. Just use bridle but build a different workflow system? I love it.
>
> Yes use the notes repo. Agent would push that repo and set it up in bridle.
>
> I think it would work. Meta notes has most of what we need. Recurrence is missing.

- **Q1: bridle with a different workflow.** No new code repo: bridle runs it, with its own
  workflow (roles, rules, skills for life admin instead of software), kept in bridle's
  `workflow/` like the other packs.
- **Q2: the notes repo becomes a bridle project.** The agent sets it up in bridle and commits and
  pushes the notes repo itself. That's the human's approval for the agent's own writes to this
  repo under the existing-projects rule.
- **Q3: meta-notes has most of it; recurrence is missing.** Recurring tasks and reminders
  (maintenance) are a meta-notes feature, filed in meta-notes' own `tickets/`.

Still open: Q4 (how due reminders reach the human), Q5 (which machine; the NUC suggested), and
Q6's branch (straight to `main`, or a `bridle-adopt` trial first per 63rv; the advisor suggests a
short trial for the setup commit only, then `main`).

## More answers, and recurrence (2026-09-30)

The human, verbatim:

> Also exact time for things. But it could be added easily.
>
> The nuc yes
>
> I need to recited the design for recurrences. An agent can make a proposal. Or you can
> research. I use an obsidian plugin that supers this. In like three syntax of it.

(Read as: meta-notes also lacks exact times, easily added; "I need to revisit the design for
recurrences ... an Obsidian plugin that supports this. I like the syntax of it.")

- **Q5: the NUC.**
- **Exact times** (a reminder at 3 PM, not just a date) are a second small meta-notes gap.
- **Recurrence: an agent proposes the design**, modelled on the syntax the human likes.

**What the human uses today (advisor, checked):** the **Obsidian Tasks** plugin
(`obsidian-tasks-plugin`), in the iCloud vault `Zettel-1`: `Personal/Home Maintenance.md` (about
130 recurring lines, most completed) and `Areas/Templates/Repeat Monthly.md`. The vault also has
dataview, templater, calendar and kanban. Real lines:

```
- [ ] replace whole house water filter 🔁 every 3 months 📅 2026-07-01
- [ ] Buy 4 bags of salt 🔁 every month when done 📅 2025-07-24
- [ ] check attic for mice 🔁 every 2 weeks when done 📅 2024-12-09
```

Rules in use: `🔁 every [N] day|week|month` (3 months, 2 and 4 weeks, month, week), with or
without `when done`, and a `📅` due date. Completing one stamps `✅ <date>` and writes the next
occurrence as a new open line: from the due date, or from the completion date with `when done`.
Only `📅` and `✅` appear; no `⏳` scheduled or `🛫` start. The Tasks plugin has no time of day, so
exact times are an extension.

**The proposal (for meta-notes):** recurrence in meta-notes' task lines, compatible with that
syntax (so the human's existing maintenance list could move over as is), plus a time of day,
e.g. `📅 2026-10-01 15:00` or a separate `⏰ 15:00` (the proposal picks one). It covers: what
`meta-notes task done` does to a recurring line, which rules to support in v1 (the ones above,
then `on Monday`, `every weekday`, and so on), and how the life assistant reads what's due.
Filed as a meta-notes design task in meta-notes' `tickets/`, for the human's review before any
build.

Still open: Q4 (how due reminders reach the human) and the notes repo's branch.
