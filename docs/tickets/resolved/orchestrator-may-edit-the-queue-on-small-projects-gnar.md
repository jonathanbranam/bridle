---
id: gnar
title: The orchestrator may edit the queue on a small project with no product manager
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [w2hj]
closed: 2026-09-30T21:53:33Z
---

## Decided (2026-09-30)

The human paused this after the first cut landed (82c42db, br-5c29) and discussed it with the
advisor. Options were: the orchestrator acts as PM (as landed), the manager plans when there's no
PM, or no queue on small projects (the empty queue reads as one tier). The human, verbatim:

> ok, yes, (1) is fine; I think orch takes over for PM is good; I dont' want to bloat orch
> responsibilities but it seems liek a reasonable tradeoff. there's a new ticket also that nobody
> is waking the manager and the manager should be woken when the queue changes - this is good; I
> think we just keep a queue I don't see a downside to it; one way of working is fine; all tasks
> can be the same tier if orch approves or whatever makes sense.

So the landed cut stands: with no PM, the orchestrator is acting PM and follows rule
`planning-the-queue`. Every project keeps a queue (one way of working); on a small project a
single tier is fine. Waking the manager on a queue change is
[[the-daemon-tells-the-manager-when-the-queue-changes-f5ww|f5ww]].

## Manager prompt (2026-09-30)

`workflow/base/roles/manager.md` said "with no product manager, noticing [open tasks] is your
job" but forbade the manager from planning or queueing them. Fixed at the human's request (via
the advisor): with no PM, the orchestrator is acting PM, and the manager tells it about open
tasks instead.

## The ask

On meta-notes (run from the NUC, no product manager), the NUC's orchestrator tried
`bridle queue set` and got `forbidden: only the product manager or the human may edit the queue`,
so the human ran it. The human, verbatim (2026-09-30, relayed by the NUC's orchestrator, m-2735):

> so, we could start a pm here, but we hardly need one for this level of work. I ran the command
> to queue the work up, but this is a feature request: at the orch and humans discretion, when
> the amount of work is small, orch can schedule work; edit the queue. we don't need to spin up a
> pm on every small project, IMO.

## Today

`require_pm_or_human` in `crates/bridle-daemon/src/server.rs` is an allowlist: `human` and an
agent with role `product-manager`. `external:orchestrator` is neither.

## Related

[[small-projects-start-without-a-manager-w2hj|small projects start without a manager]]: the
same direction, fewer standing roles on small projects.

## Acting PM needs the PM's planning guidance (2026-09-30)

On meta-notes the NUC's orchestrator then filed six ordered tasks with the order only in their
bodies, so all six showed startable until the human asked why; it added the edges with
`bridle dep add <task> --blocked-by <other>`. The human, relayed (m-2736): "that's minor; we
might need a guide or workflow for you to reference if you are \"acting pm\"." So when the
orchestrator acts as PM on a small project, it gets the PM's planning guidance (plan, dependency
edges, queue tiers), from `workflow/base/roles/product-manager.md` or a shared excerpt of it.

## Resolution

Landed in 82c42db (br-5c29) and 290c7b4 (the manager prompt). The design lives in
`workflow/base/rules/planning-the-queue.md`, `workflow/base/roles/orchestrator.md` ("Acting PM on
a small project"), `workflow/base/roles/manager.md` and `docs/design/roles-and-lifecycle.md`.
Waking the manager (or, with none running, the orchestrator) on a queue change is f5ww's own
work (br-08a6), not a follow-up of this ticket.
