---
id: j479
title: The task queue lives in bridle task, not in messages
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [f401a7f]
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

- The product manager manages the queue in bridle, not by messaging the development
  manager.
- The queue survives a crash or renewal: it's in bridle's records, not in an agent's
  context or message history.
- From bridle, the human can see the tasks being worked now and the next tasks up.

The human, verbatim, later the same day (discussing how tasks get ordered):

> dependencies should be for actual task dependencies, not for task ordering by priority;
> should the task order be written into the task? Feels like the task queue is a separate
> thing.
> manager should be mechanic not making priority decisions, that is good.
> we should formalize how tasks get ordered;
>
> should the manager have a set of equally ranked tasks and decide what to do based on
> load and available workers? Or is that the PM?
>
> I don't mind if the manager picks from equally ranked tasks that is reasonable, but the
> project plan overall should be set by the PM.

The human agreed to the model below ("yes").

## The model

- **Tasks are the what**: brief (in the task body), kind, and real dependencies
  (`blocks` edges). The product manager writes them. No ordering on the task, and no
  edges used to force an order.
- **The queue is the plan**, a separate record owned by the product manager: an ordered
  list of tiers, each tier a set of equally ranked task IDs; tier 1 before tier 2. A task
  not in the queue is backlog.
- **The development manager is mechanical.** It takes from the highest tier that has a
  startable task (dependencies met, unclaimed), and within a tier picks by load and free
  worker slots (model size, tasks touching the same files run one after another). It
  never moves tasks between tiers. If a tier is stuck on dependencies, it takes from the
  next tier rather than idling. A strict order is one task per tier.
- **Durable**: the queue lives on the state branch beside the tasks, so `bridle rebuild`
  restores it, and each change is a commit (who reprioritised what, and when). Who is
  working which task should be durable too; today claims are SQLite-only (storage.md,
  "claims"), so a rebuild forgets them.
- **Visible**: `bridle queue` shows the tasks being worked (with the worker), then the
  tiers in order.
- **Who edits it**: the product manager, and the human to override. The development
  manager only reads and claims.
- **Role prompts**: `product-manager.md` and `manager.md` change to match. No `Prepared
  task` messages; a message to the manager is only a nudge ("the queue changed"), never
  the only record.

## Resolution

Resolved by f401a7f: the task queue lives in `bridle task` (PM-owned tiers, `bridle queue`, `bridle ready`) instead of messages.
