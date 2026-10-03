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
2. Raise the wake cap from 25 minutes to at least 90 (or about 2 hours, below the background-task
   limit), in the daemon and the CLI, with the API doc.
3. Until then, the advisor uses the current cap, 25 minutes (`--timeout 1500`).
