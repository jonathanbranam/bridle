---
id: ckvz
title: Tasks split from another inherit its watchers, across projects too
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [xxxq, cy2v, 3haz, jrm2, ehv6]
tasks: [br-ckvz]
---

## The ask

The human, verbatim (2026-10-04, via advisor doc-review), after the advisor was told about the
bridle-ui halves of its tasks only by manager messages, or not at all:

> So I think what should happen here is that since you filed those tickets and asked for them to
> be worked, you should be added as a watcher to the tickets. The, sorry, you should be added as
> a watcher to the tasks. And so whenever the tasks change state, you get a message. It's not the
> responsibility of any orchestrator to, to remember to send you a message. The system should be
> sending you a message.

## What's there now (advisor, checked 2026-10-04)

- Watchers exist ([[task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq|xxxq]]).
  The filer of a task is a watcher, and it's woken on every state change.
  br-jrm2, br-wjhp and br-ehv6 have `external:advisor/doc-review` as their watcher, and those
  messages arrived.
- **The gap: a task split from another doesn't inherit its watchers.** br-jrm2's UI half
  (`ui-nprk`) and br-ehv6's (`ui-kcgy`) were made by the orchestrator in the bridle-ui project.
  Their watchers were `external:orchestrator, agent:manager-1` only. The link to the parent is
  prose in the body ("Part B of bridle task br-jrm2"). The advisor heard nothing about them
  until it went looking, and then ran `bridle task watch` on both by hand.
- **Cross-project:** the split tasks live on another daemon (bridle-ui). A waiter
  (`bridle agent wake`) watches one daemon, so watching a task there doesn't wake a session
  waiting on the bridle daemon. That's
  [[one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v|cy2v]] (`--all-projects`,
  not built).

## What it asks for

1. **A task split from or made for another task inherits that task's watchers**, in the same
   project or another one. Whoever asked for the work hears about all of it, without anyone
   remembering to tell them.
2. That needs a real link from the new task to the one it came from (e.g. `bridle task new
   --from <task>` or `--split-of`), not prose in the body. The new task also shows on the
   parent's thread.
3. Watching a task in another project must reach the watcher's waiter. Either cy2v lands, or
   the watch notification goes to the watcher on its home daemon. That's the planner's choice,
   and it may fold into cy2v or 3haz (cross-daemon mail).
4. The orchestrator's and managers' split steps use the link (role prompt or skill).
