---
id: p4xd
title: "`task done` warns about a missing summary on the human's to-dos"
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The ask

The human (2026-10-01), on finishing their own to-do: "getting a lot of these warnings on tasks":

```
% bridle task done br-b647
warning: br-b647 has no summary; record one with `bridle task summary br-b647 --text ...`
```

`task_done` (`crates/bridle/src/commands/task.rs`, after `done_task`) warns whenever
`task.summary` is unset. The summary is the worker's account of how something was implemented.
A human to-do (`task new --for-human`, claimed by `human`) has nothing to summarise, so for the
human the warning is noise.

## Fix

Skip the warning when the task's `claimed_by` is the human, or more generally when it isn't an
agent. Keep it for agent-claimed work, where it matters. Small CLI-only change, with a test.

## Done when

`bridle task done` on a human to-do prints no warning; on an agent's task it still does.
