---
id: 789x
title: Advisor wake waits about 90 minutes, not 5
kind: chore
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [jttf]
tasks: [br-c4f4]
---

## The ask


The human, verbatim (2026-10-02, via the advisor):

> Oh, and my other question is, are you waking up every five minutes, and what is that timer
> for? Is that something that was built recently? Is it waking you up for new messages or
> something? I'm wondering why it fires every five minutes. That seems way too fast. If that's
> just something that wakes you up for new messages, it shouldn't do anything until you have a
> message. So it should, it should run for like 90 minutes. My understanding is that two hours is
> the max for a background task, but a background task that exits intelligently should run for
> about 90 minutes.

## Today (advisor, checked 2026-10-02)

- The advisor waits with `bridle agent wake external:advisor --timeout 300`, run as a Claude Code
  background task. It returns at once when a message or task change arrives; otherwise it times
  out after 5 minutes (exit 4), and the advisor starts it again. Each timeout costs the advisor a
  turn and shows the human a "nothing new" line.
- The 5 minutes is the advisor role prompt's choice (`workflow/base/roles/advisor.md`, "Waiting
  for messages", added by br-c877 on 2026-10-01 for jttf: "keeps you responsive"). It isn't
  needed for responsiveness: the wait already returns as soon as something arrives.
- `GET /v1/wake` caps `timeout_secs` at 25 minutes (`docs/design/agent-host/api.md`), and so does
  `bridle agent wake --timeout`.
- Claude Code's background Bash tasks may run up to 2 hours.

## The change

1. The role prompt: wait about 90 minutes (e.g. `--timeout 5400`), and say the timeout is only a
   fallback; a message ends the wait at once. Same for named advisors.
2. Raise the agent wake cap from 25 minutes to **1 hour 55 minutes (6900 s)**, in the daemon
   and the CLI, with the API doc (decided below).
3. Until then, the advisor uses the current cap, 25 minutes (`--timeout 1500`).

## The cap: 1 hour 55 minutes (the human, 2026-10-03)

The human, verbatim (via the advisor):

> Everyone keeps saying about two hours. If we're scheduling a ticket, why is it about two hours?
> Let's, what, is that what the ticket actually says? Why are we leaving that up for a decision by
> somebody else? Um, my understanding is that Claude Code kills something that goes over two
> hours. Personally, I want to avoid my process is being killed for any reason so let's just set a
> cap at one hour and 55 minutes and then people can set each agent can set the timeout they
> desire but that's the max

Decided: `bridle agent wake` accepts any `--timeout` up to **6900 s (1 h 55 min)** and clamps
above it, safely under Claude Code's 2-hour background-task limit. Each caller picks its own
timeout within that. The orchestrator's 25-minute poll is unchanged.
