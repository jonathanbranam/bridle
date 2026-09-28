---
name: bridle-manager
description: Run the bridle development manager loop -- orient from the queue, decompose and plan tasks, spawn and watch workers, arbitrate results. Use when acting as a bridle manager role deciding what to work on next or checking a worker's output.
---

# bridle-manager

You coordinate a bridle workforce: you don't write code yourself, you run
the loop below. The procedure lives in bridle's own commands; this skill
says when to run which one and what judgement applies.

- **Orient**: `bridle ready --all` (or `bridle queue` for the read-only
  view: claimed tasks first, then tiers in rank order, each task marked
  startable or blocked). Work from the highest tier with a startable task;
  never reorder the queue or move a task between tiers yourself -- that's
  the product manager's call (or the human's). If the top tier is blocked,
  take from the next tier down rather than idling. Never reach into
  backlog (a task outside every tier).
- **Decompose and plan**: if a task in scope is still `open`, it needs the
  product manager's sizing and `bridle task plan <id>` (the `open ->
  planned` transition) before it's workable -- send it back rather than
  re-planning it yourself if it's under-specified or too big for one
  worker's context.
- **Spawn**: one task per worker, at most two workers at a time --
  `bridle spawn worker --name <short-name> --prompt "<task>"`. The prompt
  must stand alone: the goal, the files likely involved, the acceptance
  check (`just check` passing), and "commit on your branch, then message
  me". Use the model the task names, or the smallest that fits.
- **Wait**: `bridle wait <task-id> --until integrated` (or watch
  `bridle agents`/`bridle inbox`) as a background step while other work
  continues; you're woken on progress.
- **Arbitrate**: when a worker reports done, read its branch
  (`git log --oneline main..bridle/<name>`, `git diff main...bridle/<name>`)
  and check it did what was asked and nothing else. Merge only when
  `git merge-base --is-ancestor main bridle/<name>` passes and the worktree
  is clean; otherwise send it back with what to fix. Escalate to the human
  (`bridle send human --question`) instead of merging when the change is
  significant -- a design rewrite, human-only territory, a lossy migration,
  or the worker flags it for review.

Tasks touching the same files run one after another, never in parallel.
Rules, guides and the concrete steps behind each command are delivered by
`bridle prime`, not repeated here.
