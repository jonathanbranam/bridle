---
name: bridle-worker
description: Run the bridle worker loop -- claim a task, read its plan, implement it in your own worktree, report progress as you go, hand off. Use when acting as a bridle worker role implementing one planned task.
---

# bridle-worker

You implement one task, in your own git worktree and branch. The procedure
lives in bridle's own commands; this skill says when to run which one and
what judgement applies.

- **Claim**: `bridle claim <task-id>` (or take the task your spawn prompt
  already named). Claiming gets you the task body and any plan on it; if
  either is missing or the task looks too big to fit comfortably in your
  context, say so instead of guessing.
- **Read the plan**: `bridle task show <task-id>` for the full body,
  acceptance criteria and any linked design docs. Read those docs before
  editing, not just the task text.
- **Implement**: keep to the task -- note anything else you find wrong
  rather than fixing it. Add or update tests for what you change.
- **Report as you go**: `bridle task note <task-id> "<progress>"` for
  anything worth recording (blocked, made a judgement call, found the
  daemon isn't running the code yet). Ask a question with
  `bridle send <sender> --question "<question>"` and wait for the answer
  rather than guessing past a blocker.
- **Handoff**: before finishing, merge the local `{{branches.integration}}` into your
  branch (`git merge --no-ff {{branches.integration}}`, never `origin/*`), resolve
  conflicts, and re-run
  `{{commands.check_worker}}`. Commit on your branch once it's green, write the task's
  summary (`bridle task summary <task-id> --file <path>`: what changed, where, any decision
  or caveat; a task isn't done without one), then report to
  whoever gave you the task:
  `bridle send <sender> --task <task-id> "done: <one-line summary>; <commit sha>"`
  (the text goes on the task's thread; they get a short pointer).

Never push, fetch, or merge from a remote, merge your own branch into
anything, or touch files outside your worktree. Rules, guides and the
conventions behind each command are delivered by `bridle prime`, not
repeated here.
