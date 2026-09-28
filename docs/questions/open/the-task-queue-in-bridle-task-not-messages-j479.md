---
id: j479
title: The task queue lives in bridle task, not in messages
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [kp3f, a7h3, tx3f]
---

## What happened

Filed by the advisor. The human asked how work is sequenced, and how it moves from the
product manager to the development manager.

Today it moves by message. `.bridle/roles/product-manager.md`:

> **Keep the development manager's queue full.** Send it prepared tasks in priority
> order, two or three ahead of what's running: `bridle send <manager> "Prepared task
> <n>: <brief>"` (`bridle agents` shows the running manager's name). When P0's task
> records are live (`bridle task`, `bridle ready`), record tasks there instead.

The orchestrator told pm-1 "From now on bridle task is the queue" (m-0457, 2026-09-28
04:45Z) and repeated it at each renewal, but briefs and priorities still went by message
(e.g. m-0883, a7h3 put ahead of P2, 15:10Z). On 2026-09-28 ~15:15Z, `bridle task list`
had 54 open and 26 dropped tasks and nothing `planned` or claimed, so `bridle ready`
printed "no ready tasks". The task records carry no priority or order
(`tasks(id, title, kind, state, created_at, updated_at)`, `docs/design/storage.md`), and
claims are never mirrored to the state branch.

The human, verbatim (2026-09-28):

> yes, let's get more formal with the role there; some reasoning:
>
> 1. sending a message to set priority is ephemeral at best
> 2. there is no verifiable task queue to revive in case of crash
> 3. I can't query the task queue externally; I should be able to see, in bridle:
>   - current tasks being worked
>   - the next tasks up in the queue currently
> 4. it's correct for the pm to manage the task queue, but not through messaging.

## What the human wants

- The product manager manages the queue in `bridle task`, not by messaging the
  development manager.
- The queue survives a crash or renewal: it's in bridle's records, not in an agent's
  context or message history.
- From bridle, the human can see the tasks being worked now and the next tasks up.

## The advisor's recommendation

- Use the lifecycle that's already built (`docs/design/roles-and-lifecycle.md`): the
  product manager writes the brief into the task body and moves it `open -> planned`,
  with `blocks` edges where order matters; the development manager takes work from
  `bridle ready` and claims it for the worker it spawns (`claimed_by`).
- Add the one missing piece, an order among ready tasks (e.g. a priority or rank
  field the product manager sets), so "next up" is a query, not a message.
- One view for the human, e.g. `bridle queue` or `bridle task list --queue`: claimed
  tasks (with the worker), then ready tasks in order.
- Change both role prompts: no `Prepared task` messages. A message to the manager is
  only a nudge ("the queue changed"), never the only record.

## Notes

- Claims live only in SQLite (storage.md, "claims"). After `bridle rebuild`, what was
  being worked would be lost; whether that matters is part of point 2.
