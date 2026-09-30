---
id: tr7k
title: A task records its branch, merge commit and an implementation summary
opened: 2026-09-29
resolved: 2026-09-29
repos: [bridle]
changes: [a602652]
specs: []
needs: []
see: [sq4m]
---

## The ask

The human's words are quoted in [[squash-merge-each-task-sq4m|sq4m]]: the task should show
the branch that did it (which is then deleted), the SHA of the merge that implemented it, and
a summary of how it was implemented.

## Why: traceability

The human, verbatim (2026-09-29):

> I want the advisor to be able to answer a query such as: What was done to implement the
> ticket for vim time block highlighting? The advisor should be able to find the ticket which
> has a note about the implementation; a summary; and then trace back to the merge commit
> easily to see what files changes instead of grepping the codebase and looking at the recent
> git log - it should be a precision inspection of what changed. E.g. if that were an openspec
> change we would have a proposal, design, spec updates, and tasks PLUS the code changes to
> inspect to see what was done. We're missing all of that. I don't think we *need* all of
> that necessarily, but we need the traceability

**Acceptance:** from a task id alone (any project; the advisor reads other projects' daemons
without a token, 9c63), one command shows the brief (the task body), the implementation
summary, the branch and the merge commit, and `git show --stat <commit>` gives exactly the
files that task changed. With sq4m's one-commit-per-task that's precise; with `--no-ff` merges
it's `git diff <commit>^1 <commit>`. Searching tasks by words ("time block") must find it,
title or body.

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

## Resolution

Resolved by a602652: `bridle task done --branch` and `bridle task summary` record branch, commit and summary on the task; `task show` prints them (docs/design/cli.md, storage.md).
