---
id: prvy
title: What happens to the daemon and agents when the laptop sleeps or loses the network
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [where-the-single-orchestrator-lives-hj4g]
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
  can't reach it from the phone meanwhile (hj4g).

`bridle budget hold [--for | --until]` already exists as a deliberate pause.

## Notes

Spike: observe a real sleep and wake with a worker mid-turn (events, agent states,
the governor's state, whether work carries on without a hand), and record what needs
fixing. The first observation is the human's commute on 2026-09-28.
