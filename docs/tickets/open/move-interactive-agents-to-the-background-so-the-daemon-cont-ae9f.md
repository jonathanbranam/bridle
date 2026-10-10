---
id: ae9f
title: Move interactive agents to the background, so the daemon controls their message delivery
kind: feature
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

The human, 2026-10-10 ~10:15 AM ET (to the aide, relayed verbatim to the PdM):

> Another point in our product plan here is that we need an epic around getting more of these
> interactive agents moved to the background so that this old waiting message delivery thing
> becomes something we have a lot more control over. I assume, and perhaps I'm wrong here, that
> background processes like the manager, the project manager, and the workers don't have to do
> this waiting-for-wakes thing.

And on the waiter started with `&` (ticket 9aj2):

> One of the considerations here is whether the way we're using the Bridle CLI is the best way to
> do it, or whether we can use a tool call. Can we add a tool for Bridle? This is a future idea
> question: would that make things more consistent, and would that fix this waiting problem?

## What this epic is for

Today the orchestrator, the aide and the advisors are interactive Claude Code sessions. They
learn of new messages only through a `bridle agent wake` waiter they start as a background
command. Each one has started a waiter wrongly at least once (`&`, output discarded), losing
messages (ticket 9aj2). Agents the daemon spawns (managers, pm-1, workers, designers) are headless
stream-json sessions: as far as the PdM knows, the daemon delivers their messages as turns and
they run no waiter. The designer should confirm this from `docs/design/agent-host/`.

The goal: more of the interactive roles run under the daemon in the background, so the daemon,
not the agent's own discipline, controls message delivery. The human talks to them through the
web UI, mail or a thin front end.

## Related

- z485: make the orchestrator non-interactive (recorded 2026-10-03, not built). The human's
  to-do br-3a42 asks them to expand on it.
- r8kv: seats, splitting the advisor; gtzx: every role is a named, tracked seat (held for the
  human's review).
- 9aj2: message delivery you can check (in the messaging epic; its part 1 is planned).
- The future-idea ticket for a bridle tool instead of the CLI (filed by the aide).
- kuw2: one daemon per machine. The human, 2026-10-10: "that would entirely rewrite message
  delivery", so this epic's design should say how it depends on that decision.

## First step

A designer pass: which roles can move, what the human loses and gains (talking to the
orchestrator directly today), how the human reaches a background role, and an order. Not
scheduled until the PdM places it after scheduled messages.
