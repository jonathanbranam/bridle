---
id: sq4m
title: Land each task as one squash commit carrying the task's info
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [tr7k, m2fq]
---

## The ask

The human, verbatim (2026-09-29):

> the bridle ticket should indicate the name of the branch (which should be deleted) and the
> SHA of the merge that implemented the task. Are merges squashes from the workers? They
> should be; main (or the merge-target branch) should be a series of merges from completed
> tasks with info about the task in the merge commit. Tasks should have a summary of how they
> were implemented as well;
>
> Add these as tickets for the worker and manager agents to follow in their instructions.

This ticket is the merge half; [[task-landing-record-tr7k|tr7k]] is the task-record half.

## Today

They aren't squashes. The manager runs `git merge --no-ff bridle/<name> -m "Merge
bridle/<name>: <summary>"` (`workflow/base/roles/manager.md`, "Merge completed work"), so the
integration branch gets the worker's whole branch: its own commits, plus a "Merge main into
bridle/<name>" commit each time it caught up (m2fq). The merge message names the branch, not
the task.

## What to change

- **One commit per task on the integration branch.** The manager lands a checked branch with
  `git merge --squash bridle/<name>` and one commit whose message carries the task:
  - subject: `<task id>: <task title>`;
  - body: the worker's implementation summary (tr7k), then `Task: <id>` and
    `Branch: bridle/<name>` trailers (plus the usual co-author trailer).
  The integration branch's history then reads as a list of completed tasks.
- **Checks move accordingly.** Keep the worker's `git merge --no-ff <integration>` catch-up
  and `{{commands.check}}` on its branch (they're what makes the squash apply cleanly and
  green). The manager's `merge-base --is-ancestor` gate stays; after the squash, the
  conflict-marker check and the push stay as they are.
- **Branch deletion.** A squashed branch isn't an ancestor of the integration branch, so git
  won't call it merged: `bridle rm <name> --delete-branch` must delete it anyway (check it
  uses `-D`, or deletes after confirming the squash commit exists).
- **Update the instructions**: `workflow/base/roles/manager.md`, `workflow/base/skills/manager/SKILL.md`,
  `workflow/base/roles/worker.md` and `workflow/base/skills/worker/SKILL.md` (the worker writes
  the summary the commit uses), and `docs/design/agent-host/operating-model.md` ("Merging
  completed work").
- Applies to every project (bridle, meta-notes, track-web): the project layers carry their
  own manager prompts in meta-notes (`.bridle/roles/manager.md`), which need the same change.

## Notes

- The orchestrator's reading of "a series of merges ... squashes": one squash commit per task.
  If the human meant a `--no-ff` merge commit over a single squashed commit (a merge bubble
  per task), only the manager step differs.
