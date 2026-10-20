---
name: bridle-worker
description: Run the bridle worker loop -- claim a ticket, read its plan, implement it in your own worktree, report progress as you go, hand off. Use when acting as a bridle worker role implementing one planned ticket.
---

# bridle-worker

You implement one ticket, in your own git worktree and branch. The procedure
lives in bridle's own commands; this skill says when to run which one and
what judgement applies.

- **Claim**: `bridle ticket claim <ticket-id>` (or take the ticket your spawn prompt
  already named). Claiming gets you the ticket body and any plan on it; if
  either is missing or the ticket looks too big to fit comfortably in your
  context, say so instead of guessing.
- **Read the plan**: `bridle ticket show <ticket-id>` for the full body,
  acceptance criteria and any linked design docs. Read those docs before
  editing, not just the ticket text.
- **Implement**: keep to the ticket -- note anything else you find wrong
  rather than fixing it. Add or update tests for what you change.
- **Report as you go**: `bridle ticket comment <ticket-id> "<progress>"` for
  anything worth recording (blocked, made a judgement call, found the
  daemon isn't running the code yet). Ask a question with
  `bridle send <manager> --question "<question>"` and wait for the answer
  rather than guessing past a blocker.
- **Handoff**: before finishing, do the following in order:
  1. **Check docs** (rule `docs-current`): if your change affects behaviour the
     project's docs describe (design docs, briefs, CHANGELOG), update them in your
     working tree. Include these edits in the commit so they travel with the code.
  2. **Merge and check**: merge the local `{{branches.integration}}` into your
     branch (`git merge --no-ff {{branches.integration}}`, never `origin/*`), resolve
     conflicts, and re-run it as `{{commands.check_worker}} > /tmp/<task>-check.log 2>&1` and judge by the exit status alone; read the log's tail only on failure, and on success only its nextest `Summary` line, checking the count is not 0 and inside the band (half to double) of `$BRIDLE_WORKSPACE/last-full-test-count`.
  3. **Commit and report**: once it's green, write the ticket's summary (`bridle ticket summary
     <ticket-id> --file <path>`: what changed, where, any decision or caveat; a ticket isn't
     done without one). Include 'docs: updated X' or 'docs: none needed' in the summary,
     then report to your spawner (never `human` unless it says to: a one-off worker
     reports to the manager, found with `bridle agent list --json`):
     `bridle send <manager> --task <ticket-id> "done: <one-line summary>; <commit sha>"`
     (the text goes on the ticket's thread; they get a short pointer).

  Run these commands with Bash: printing them does nothing, and the manager only learns
  you are done from the send.

Never push, fetch, or merge from a remote, merge your own branch into
anything, or touch files outside your worktree. Never kill processes by name
(rule `no-kill-by-name`). Rules, guides and the conventions behind each
command are delivered by `bridle orchestrator prime`, not repeated here.
