---
id: n8tj
title: Talk about a task on its thread; messages only notify
opened: 2026-09-29
resolved: 2026-09-29
repos: [bridle]
changes: [6cae4af]
specs: []
needs: []
see: [tr7k, sq4m]
---

## The ask

The human, verbatim (2026-09-29):

> When discussing a task with the manager / worker or you, do they message directly? or add
> notes to the ticket and then send a message "you have a note" or what? Like when humans are
> doing software dev, if we have DMs with each other, the information about the conversation
> is lost. Instead we use JIRA and make comments on the ticket so the information is associated
> with that ticket after completion. Let's use this principle.

## Today

Almost all task talk is direct messages (`bridle send`): briefs, "done: <sha>", review
findings, "send it back", questions. Tasks have threads (`bridle task note`, `ask`, `answer`)
but they're used little, apart from the landing summary (tr7k).

## Done now

- Base rule `workflow/base/rules/talk-on-the-task.md` (must, every role): anything about a
  task goes on its thread; a message only notifies and names the task.

## To build

- **One command for both**: `bridle send <agent> --task <id> "..."` (or `bridle task note
  <id> --notify <agent>`) writes the text to the task's thread and sends a short
  notification that points at it, so agents can't do one without the other.
- **Messages that name a task are linked to it**: a message with `--task` shows in
  `bridle task show <id>`'s thread.
- **Role prompts and skills** (manager, worker, product manager) updated to use it: the brief
  on the task, done reports and review findings as task notes.

## Resolution

Resolved by 6cae4af: `bridle send --task` and `bridle task note --notify` put the talk on the task's thread; messages only notify.
