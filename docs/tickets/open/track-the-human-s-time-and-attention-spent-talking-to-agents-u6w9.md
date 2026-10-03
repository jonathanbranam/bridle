---
id: u6w9
title: Track the human's time and attention spent talking to agents
kind: feature
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq]
tasks: [br-d772, br-u6w9]
---

## The ask


The human, 2026-10-01, verbatim:

> file a ticket for work that I'd like to get started today -
>
> tracking the human's time and attention in bridle
>
> I want a system (can be basic to start with) that tracks how much time the human spends
> interacting directly with bridle agents. Basic reporting is ok here - keep track of messages that
> arrive at interactive bridle agents (maybe they're in events already? IDK). the data collection
> part should be started immediately. the result should be that we can build a report of the
> human's time: daily, weekly, typical hours, stat/stop etc. things like that. I want to know how
> much of my time is spent talking to agents.
>
> We don't need to track things outside of bridle or things the system cannot reasonably find out.
> Let's see... so interactions with agents is the main and primary one; we could also track if the
> human uses the CLI to interact with tasks or in other ways; KISS here though, the main, big big
> thing is: how long do I spent sitting here talking to agents.

Priority: start today. Collection first, because history can't be recovered later; the reports
can follow.

## Today

- The human talks to agents mostly by typing into interactive sessions: the orchestrator and
  the advisors, started by `bridle session`. These sessions run outside bridle, so the human's
  prompts reach no bridle event or message.
- `bridle session` already gives both roles a `UserPromptSubmit` hook, `bridle focus gate`
  (cvaq, `crates/bridle/src/session.rs:22`). It runs on every prompt the human types in those
  sessions, so it is one place a timestamp per prompt could be recorded.
- What the human sends through the CLI (`bridle send`, `bridle task ...` as `human`) already
  reaches the daemon as messages and events.

## Wanted

- **Collection, now:** a record of each time the human interacts: when, which session or agent,
  which machine. Interactive sessions first; the human's CLI calls are optional.
- **Reports, later:** the human's time talking to agents, by day and by week; usual hours; when
  each day started and stopped.
- Out of scope: anything outside bridle, or anything bridle can't reasonably find out.
- **Across machines, later:** the human, 2026-10-01, verbatim: "there should be some way to
  consolidate reports between machines as well, but that can be a TBD / TODO for later; for now
  we can do that consolidation manually without issue." So each machine records its own; merging
  them is manual for now.

## Collection built (br-0297)

Every prompt the human types in a `bridle session` (orchestrator, advisors, Remote Control
included) appends `{"at","machine","project","role","session"}` to `~/.bridle/prompts.jsonl` on
that machine (`roles-and-config.md`). On dalek, 652 lines from 2026-10-01 to 2026-10-03.

## Build it: reports and a UI (the human, 2026-10-03)

The human, verbatim (via the advisor, while mowing the lawn):

> Find or create the ticket to build tracking the humans interaction time. We jotted down some
> ideas. And we're tracking events I believe.
>
> Goal: all human interaction time with agents tracked and reported across all projects and
> machines. Data collection can be polling and delayed 5-10 minutes doesn't have to be real time
> but close ish.
>
> Once the data is collected and available expose it and build a UI portion that shows the data in
> various roll ups that I can configure. I want to see time per project, per agent, summed over
> days and weeks
>
> Also showing time spent in a single day and tracking how many simultaneous conversations I'm
> shaving actively at once. Reports for time of day interactions both over a week or weekend vs
> weekday or specific days of the week.
>
> I'm mowing the lawn so make reasonable design decisions and ask for it to be implemented today.
> Unless there is a critical decisions, get the work going.

("shaving" is "having".)

## Design (the advisor's decisions, per the human's go-ahead; adjustable later)

**Sources.** (1) Each machine's `~/.bridle/prompts.jsonl` (prompts in interactive sessions).
(2) Messages the human sends through bridle (`from = human` in each daemon: `bridle send`,
answers), as interactions with that project, agent = the recipient. Nothing outside bridle.

**Time from timestamps.** A prompt is a point in time; time is inferred:
- Within one session, a prompt counts the time until the next prompt in that session, **capped at
  10 minutes** (reading and thinking between prompts); the last prompt of a run counts **2
  minutes**. A gap over 10 minutes ends a run.
- **Human time** for any total is the **union** of these intervals, so two conversations at once
  aren't counted twice. Per-project and per-agent totals use each session's own intervals (so they
  can add up to more than the human's total; the UI says so).
- **Simultaneous conversations** at a moment = the number of sessions with an interval covering
  it. Reported per day: peak, and minutes at 1, 2, 3+ at once.
- Cap and tail are config (`[interactions] gap = "10m"`, `tail = "2m"` in the gateway's config).

**Agent** = the session's role name (`orchestrator`, `advisor`, `advisor-tickets`), per project
and machine. **Times** in US Eastern (the human's zone), days split at local midnight.

**Across machines.** Each daemon serves its machine's prompt log: `GET /v1/interactions?since=`
(human or local reads; machine-wide, so any one daemon per machine is enough). The gateway polls
every daemon it knows (this machine and others, as its listing already does) **every 5 minutes**,
dedupes by `(machine, session, at)`, and keeps the merged data in its own store. An unreachable
machine is reported, not fatal; its data catches up when it's back (the logs are append-only).

**API (gateway, `/api/v1`, ts-rs types).** `GET /interactions/report?from=&to=&group=project|agent|machine&bucket=day|week`
(totals per group per bucket, plus human total); `GET /interactions/day?date=` (the day's
intervals per session, for a timeline, and concurrency); `GET /interactions/hours?from=&to=&days=weekday|weekend|mon,tue,...`
(minutes per hour of day, averaged over the matching days). Raw intervals too, so the UI can roll
up its own way.

**UI (bridle-ui, a "Time" page).** Pick a range (this week, last 4 weeks, custom) and group by
project, agent or machine:
- totals per day and per week (stacked bars), with the human's total line;
- one day: a timeline of each session's intervals, with simultaneous conversations shaded, and the
  day's start, stop, total and peak concurrency;
- time of day: an hour-by-hour chart, filterable to weekdays, weekends, or chosen days of the week.
Choices persist in the browser.

**Out of scope for today:** the human's CLI reads (`bridle status` etc.), anything outside bridle,
backfill before 2026-10-01.
