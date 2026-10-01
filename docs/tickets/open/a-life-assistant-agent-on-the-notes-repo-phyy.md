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

## Survey and recommendation (br-ca8a, 2026-09-30)

Read-only survey: `meta-notes --help` and the meta-notes checkout's `tickets/` (nothing run
against the real notes repo, nothing written to it or to meta-notes). The recurrence design
proposal already exists as meta-notes `tickets/recurrence-and-time-of-day.md` (mn-ba09, awaiting
the human's review); this survey depends on it and does not redo it.

### Verbs against the CLI

| Verb | meta-notes today | Gap |
|---|---|---|
| Create a task or reminder | none: `task update` edits an existing checkbox line; no `task add` | **`task add`** (mn-ba09 open question 6). Until then the agent would hand-edit markdown, which the CLI's conventions are meant to prevent |
| Reminder with a time | dates only (`📅`), day precision | time of day (mn-ba09 proposes `⏰ HH:MM`); "what is due now" query (`--at`) |
| Recurring maintenance | none; the human's Obsidian Tasks lines (`🔁 every 3 months`) are inert | recurrence: mn-ba09 (Obsidian syntax as is; completing spawns the next line) |
| Record an event | none as such. `note new <path>` creates a note from a template; daily/weekly notes exist | probably no CLI gap: an event is a dated line in the daily note or a note under `area/`. Needs a convention in the workflow's rules, not code |
| Remember something | `note new` (files a note anywhere, from a template); `move`, `rename`, `archive` | none for filing; a rule saying where things go (PARA) |
| Check in on projects | `projects [--warnings]`, `project brief`, `changes`, `tasks --overdue/--ready/...`, `ceremony status` | none |
| Check in on maintenance | `tasks` with `--overdue`, `--folder area/...` | works once recurrence lands; before that a done maintenance line never comes back |
| Calendar context | `calendar` (Google Calendar export) | none |

So: reads and check-ins are covered today. Capture needs `task add`, time of day and recurrence.
Events and "remember" need only rules.

### Options

**Shape (Q1): where the workflow lives.**
- A. *Life-admin pack* `workflow/packs/life-admin/` (rules, skills, a `roles/` prompt), listed in the
  notes repo's `.bridle/config.toml`. Follows the human's answer and needs no new code repo.
- B. Project layer only (`.bridle/` inside the notes repo). Puts bridle files in the human's repo
  with nothing shareable.
- **Recommend A**, plus a small `.bridle/config.toml` in the notes repo (packs, and a
  `system_prompt` pointing at the pack's role prompt). Caveat to check when building: the base layer
  (`workflow/base/`) is software-oriented (worker/manager roles, `just check`-style rules) and
  applies to every project; confirm the notes project can opt out of, or override, base rules that
  make no sense there. Per workflow-layers.md, role prompts are not overlaid, so the pack ships one
  role prompt, wired by the project's `system_prompt`.

**What the pack holds.** One standing role (the life assistant), rules such as "only edit notes through
`meta-notes`", "confirm before archiving or deleting", "capture goes to `area/` or `project/` by these
rules", skills mapping verbs to commands (capture task, capture event, remember, project check-in,
maintenance check-in, daily digest), and the reading of due items from `meta-notes tasks`.

**Machine (Q5): answered, the NUC.** It runs meta-notes and the bridle daemon (docs/context/nuc-host.md).
The notes repo needs a clone there.

**Git (Q6).**
- A. Agent commits and pushes straight to `main`. Simple; a bad edit lands at once.
- B. Agent works on a branch and the human merges. Safe; too heavy for "remind me to buy salt".
- C. One `bridle-adopt` trial branch for the setup commit (`.bridle/`, per existing-projects.md),
  then the agent commits and pushes to `main` for routine capture, one commit per change.
- **Recommend C.** Also: the agent pulls before it works and pushes after, since the human edits
  the same repo from their laptop and Obsidian; a rule to never force-push and to stop and ask on
  a conflict. Committing is cheap to undo; the notes are plain text under git.

**Reminders reach the human (Q4).**
- A. rs7p daily digest (6:30 AM) gains a "due today and overdue" section. Cheap, but a 3 PM reminder
  arrives at 6:30.
- B. Remote Control / chat: the assistant pushes a message when something is due. Needs a clock
  (a scheduled wake) and depends on the bagg spike, whether a bridle-hosted session can be
  reached remotely.
- C. Both: digest for dated items, a timed push for items with `⏰`.
- **Recommend A first**, since day-precision reminders need nothing new, then B for timed ones once
  time of day exists and bagg is answered. Timed pushes could also be an rs7p mail rather than
  Remote Control, which avoids the bagg dependency.

**How the human talks to it.** Remote Control first (as the ticket says), depends on spike bagg.
Email to a notes address (rs7p) is the fallback that works with the mail work already built.

### Dependencies (listed, not designed)

1. meta-notes mn-ba09: recurrence and time of day, awaiting the human's review.
2. meta-notes `task add` (mn-ba09 open question 6; needed for any capture).
3. bridle: a standing non-software role. The orchestrator's standing-session support (fx7x) is built
   for the orchestrator only; hosting a second standing agent for another project is a change to
   check before building.
4. Spike bagg (Remote Control on a bridle-supervised session) for chat-style delivery.
5. rs7p digest section for due reminders, if Q4 is A or C.

### Remaining open questions for the human

1. **Q4:** digest only, chat push, or both? (Recommend digest first.)
2. **Q6 branch:** approve option C (one trial branch for setup, then `main`)?
3. **`task add` now?** Approve it as the first meta-notes build ahead of recurrence, since no capture
   works without it.
4. Do you still edit the notes in Obsidian? It affects mn-ba09's time syntax (`⏰`) and how often
   pull conflicts will arise.
5. Should the assistant act unprompted (nag about overdue maintenance) or only answer when asked
   and in the digest?

## The concierge, and what bridle lacks for it (2026-10-01)

The human (via the NUC's orchestrator, m-3169), verbatim: "the notes needs a concierge-style
personal orchestrator that is unique from the 'software management' orchestrator solution we have
now. TBD on the full design and capabilities, but this is an outline."

Today the notes project is set up as bridle project `notes` on the NUC
(`/srv/shared/work/notes-work/notes`, branch `bridle-adopt`; the human's to-do is mn-7951 on
meta-notes). An advisor runs in it with `notes/.bridle/roles/advisor.md`, and the NUC's
orchestrator drives the project with `--project notes`. The gaps it found (m-3148, m-3169):

1. **A project-defined external role.** `bridle session` knows only `orchestrator` and `advisor`.
   A project's `.bridle/roles/orchestrator.md` can only append to the base orchestrator text, which
   is about running software (don't edit the clone; managers; merges). Wanted:
   `bridle session <role> --project p` for a role the project defines, whose own prime text
   replaces the base, with wakes and supervision like the orchestrator's.
2. **Several supervised sessions per machine.** The supervision files are machine-wide:
   `$BRIDLE_HOME/orchestrator.pid`, `.session`, `.exits` (`crates/bridle/src/session.rs:161`,
   `crates/bridle-daemon/src/orchestrator.rs:231`). Two daemons with `[orchestrator]` enabled
   would watch, hand over and relaunch the same session. So `notes` runs with supervision off.
   A per-project assistant needs the files named per project or per role. See ma8e (one
   orchestrator per machine).
   The human, 2026-10-01 (m-3172), verbatim: "For restart, we should likely just generalize that
   to any agent based on local configuration. It seems like something useful. The orch can launch
   advisors, but it seems reasonable to put that on bridle instead of overloading the orchestrator
   with things to do." So bridle supervises any external session a project's config names
   (orchestrator, advisors, a concierge): it launches them, restarts them on a crash and hands
   them over at the context limit. Each session gets its own pid, session and exits files.
3. **Wake on any message.** `wait-for-wake` serves `external:orchestrator` only. `--mail` returns
   only on mail from the email bridge, and `bridle wait` needs a task. The stopgap is a loop
   polling `bridle inbox --json`. Wanted: a message waiter for any external principal.
4. **Durable timers.** The concierge should act on times set in the notes (meta-notes has `⏰`
   times and `tasks --overdue --due --at now`). Today only Claude Code's `CronCreate` and
   `ScheduleWakeup` exist, and they're lost on restart. Wanted: a scheduled wake the daemon owns,
   e.g. per-project timers, or a wake source that runs `meta-notes tasks --at now --json`.
5. **Google Calendar through MCP or a connector**, read-only first. Today `bridle session` blocks
   it: `--strict-mcp-config` and `disableClaudeAiConnectors` (`session.rs`, LEAN). Wanted:
   per-role MCP config or allowed connectors (see u6wk). The fallback is meta-notes `calendar`
   from a Google export. No Gmail; email goes through the rs7p bridge.
6. **Per-project focus hours.** Filed on its own as cvaq's follow-up (below).
7. **Base rules and roles assume software** (worker, manager, check commands). A non-software
   project needs to opt out of them or override them.
8. **A daemon with no manager or workers.** `notes` sets `[roles.manager] autostart = false`,
   `resume_on_restart = false`, `packs = []`, and no check command. Verified on bridle 0.4.0
   (m-3182): the notes daemon runs with it and `bridle doctor` passes. Keep it supported.
