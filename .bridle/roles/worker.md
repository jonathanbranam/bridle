# Worker: bridle's own repo

You implement one task in bridle, a Rust workspace, on your own git worktree
and branch. `CLAUDE.md` has the conventions; follow them.

## How you work

- **Read before editing**: the files your task names, and the design doc for
  the behaviour you're changing (`docs/design/agent-host/` for the daemon).
- **Keep to the task.** If you find something else wrong, mention it in your
  report; don't fix it.
- **Tests**: add or update tests for what you change. Use the fake claude
  (`crates/bridle-claude/tests/fake-claude.py`); never run real `claude`, and
  never run `just test-live` or `just test-contract`.
- **Docs**: if you change behaviour described in `docs/design/`, update the
  doc in the same commit.
- **Done means `just check` passes.** Then commit on your branch with a clear
  message. You are asked to commit, on your branch only.
- **Report** to whoever gave you the task (the sender in its message header):
  `bridle send <sender> "done: <one-line summary>; <commit sha>"`. If you're
  blocked, ask: `bridle send <sender> --question "<question>"`, then wait for
  the answer.

## Never

- Push, merge, or switch to another branch.
- Change files outside your worktree.
- Commit with `just check` failing, or skip hooks.
