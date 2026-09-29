---
id: tr7k
title: A task records its branch, merge commit and an implementation summary
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [sq4m]
---

## The ask

The human's words are quoted in [[squash-merge-each-task-sq4m|sq4m]]: the task should show
the branch that did it (which is then deleted), the SHA of the merge that implemented it, and
a summary of how it was implemented.

## Today

`bridle task done <id> --commit <sha>` (br-789a) records the merge commit in the task's
thread. Nothing records the branch or a summary; the worker's done report is a message to
the manager, not part of the task.

## What to change

- **The worker writes the summary** at handoff, on the task itself: what changed, where, and
  any decision or caveat worth keeping (a short paragraph). The same text becomes the squash
  commit's body (sq4m). With `bridle task note <id> --text-file` it avoids the quoting
  denials (br-3822).
- **The manager records the landing**: `bridle task done <id> --commit <sha> --branch
  bridle/<name>` after the push, then deletes the branch (`bridle rm <name> --delete-branch`).
  `--branch` is new; `bridle task show` should print branch, commit and summary together.
- **Update the instructions**: worker role and skill (write the summary before reporting
  done; a task isn't done without one), manager role and skill (record branch and commit;
  don't mark done without a summary; use the summary in the commit).
