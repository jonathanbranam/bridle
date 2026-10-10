---
id: bnfs
title: Deep links to a human to-do open the task page and fail; open the to-do page instead, also when a to-do id is typed into the task page
kind: bug
opened: 2026-10-10
filed_by: external:aide
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [yfjc]
tasks: []
---

## The ask

The human, 2026-10-09 ~10:40 PM ET, verbatim (to the aide):

> file a low pri bug: the deep link in bridle ui for human-claimed tickets fails to open because it opens as a task and not a todo; the UI or backend should discover this is a human todo and open the proper page; this should also happen if the user types a todo id into the task page, instead of saying "not found" it should open the todo page and scroll the todo into view.

## Facts

- `bridle link <task id>` prints `<public_url>/task?id=<id>` for every task, a human's to-do too
  (for example br-q7ua, br-x7fx tonight); the human's to-dos are tasks claimed by `human`
  (`bridle task new --for-human`).
- Related: the unified URL scheme is task br-enx3 (planned).

## The ask

1. A link to a task that is a human's to-do opens the to-do page (the UI or the gateway works out
   that it is one), not the task page that fails.
2. Typing a to-do's id into the task page opens the to-do page with that to-do scrolled into view,
   instead of "not found".

Priority: low (the human).
