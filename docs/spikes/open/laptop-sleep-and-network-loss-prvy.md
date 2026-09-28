---
id: prvy
title: What happens to the daemon and agents when the laptop sleeps or loses the network
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What happened

The human, verbatim (2026-09-28):

> what happens when I leave for work in 20 min and have to shut my laptop lid and lose
> wifi on my way to work? This will happen at least twice a day. Do we need to plan for
> that in some way? We could restrict work in advance or, how long to pause work? Can we
> have a "take a pause" or is it OK to just let things be interrupted and test the
> restart?

## Why it matters

It happens at least twice a day, while the daemon runs on the laptop. Nothing has
tested it. Things that are likely to trip on a wake, from the design as built:

- Stall detection (`stall_after`, 10 min, docs/design/agent-host/agents.md) sees a
  long silent stretch across the sleep.
- The budget governor treats a reading older than `max_staleness` (10 min) as unknown:
  it holds, and at three times that it winds down (docs/design/usage-and-budget.md,
  "Unknown is not safe"). A wake after a commute is well past both.
- In-flight API calls from `claude` fail on the lost network; how Claude Code retries,
  and whether a turn ends in an error, is unverified.
- The orchestrator, a Claude Code session on the same laptop, sleeps too, so the human
  can't reach it from the phone meanwhile ([[where-the-single-orchestrator-lives-hj4g|where the orchestrator lives]]).

`bridle budget hold [--for | --until]` already exists as a deliberate pause.

## Notes

Spike: observe a real sleep and wake with a worker mid-turn (events, agent states,
the governor's state, whether work carries on without a hand), and record what needs
fixing. The first observation is the human's commute on 2026-09-28.

## First observation: 2026-09-28 morning commute

Lid closed 13:03 UTC, opened 13:21 (the human's times); the daemon's first event
after the gap is at 13:18:55. At the close: three workers mid-turn, governor
`normal`, last event seq 18298.

- **Nothing stopped or crashed.** No `agent.exited`; all three workers were still
  `working`, and tool calls and turn ends resumed by 13:22 with no errors seen.
- **`agent.stalled` for all three workers at 13:18:55**: the sleep counted as
  silence. A false alarm, but only an event; nothing acts on it.
- **The governor held for about 2.5 minutes**: `normal` -> `holding` at 13:19:08
  (usage reading stale), back to `normal` at 13:21:44 once a fresh reading
  landed. The wind-down (three times `max_staleness`, 30 min) wasn't reached.
- `just check` running in the orchestrator's shell during the sleep: see below
  if it failed on timing.

So a sleep of about 20 minutes needs no plan: let it be interrupted. Still to
observe: a sleep longer than 30 minutes (the staleness wind-down stops working
agents; does work come back without a hand?), and whether stall detection
should discount time the machine was asleep.

The human runs `caffeinate -si` all the time (their words: "I do that b/c the
laptop will sleep even when plugged in usually; I handle that manually"). It
didn't keep the laptop awake with the lid closed on battery: `-s` only holds
off sleep on AC power. `pmset -g log` for the commute (Eastern times) shows
it asleep, with two short maintenance dark wakes: 09:07:27 (8 s) and
09:18:56 (8 s), then the full wake at 09:21:34. The daemon's events at
13:18:55 UTC (the stall events and the governor's hold) come from that
second dark wake, not from the lid opening.
