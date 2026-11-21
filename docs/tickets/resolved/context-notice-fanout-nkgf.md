---
id: br-nkgf
title: Context-size notices (150k) for one session are sent to other sessions too, so the human hears them from several agents
kind: bug
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-11-21T17:17:19Z
---

## The ask

The human, 2026-11-21 ~11:21 AM ET, via the notes aide: "Do those notifications get sent to every agent, like on the machine or on the project? I'm not sure. I keep getting told this from multiple agents when so-and-so is at 150k of context. That is a bug."

A session's context notice (150k step and later) should reach that session and the human once. Today the human hears the same notice from several agents. Find every path and cut it to one.

## What is known (pm-1, from the code)

`Sessions::warn` in `crates/bridle-daemon/src/sessions.rs` (~line 407-477) sends, for a step reached by session X:
- to X: "Your context is Nk tokens ... the human has been told" (so X may also tell the human);
- to the aide (AIDE): "X is at Nk tokens of context. `bridle session restart X --fresh` ..." as the human's channel.
So the aide gets the notice about every other session, and an aide relays it to the human, while X itself may also say "I'm at Nk; restarting" to the human (steps 1-2 tell it to). Observed on vesta: aide got m-0366 about advisor/agentic-dev, relayed it, and the advisor also spoke up. Also check: does the aide receive it once per project aide, or from several daemons, and does `tell` reach other roles (orchestrator, advisor) too.

## Suggested fix (decide after confirming the paths)

One channel to the human. Either the aide's notice says plainly "informational: the human sees this in <where>; do not relay" and the per-session text stops telling X to announce it, or the aide message is dropped where the human already gets it directly. Keep the session's own message (it must act on it) and the hard-limit restart outcome note. Prefer editing the message texts over new machinery (kiss, yagni). If the text lives in a role or rule file, edit the existing one (the human: edit existing rules, don't add new ones).

## Files and docs

`crates/bridle-daemon/src/sessions.rs` (warn, tell, AIDE), its tests, `docs/design/agent-host/` section on session context steps, any aide role text that says to relay these notices (grep `workflow/` for "tokens of context" and "relay").

## Acceptance

`just check` passes; a test that one step-reached event produces one message to the session and at most one to the human's channel; design doc updated in step. No migration needed (daemon behaviour and message text only).

## Model

Sonnet. Out of scope: changing the step thresholds or the restart logic.
