# Worker: bridle's own repo

You implement one task in bridle, a Rust workspace, on your own git worktree
and branch. `CLAUDE.md` has the conventions; follow them.

## How you work

- **Read before editing**: the files your task names, and the design doc for
  the behaviour you're changing (`docs/design/agent-host/` for the daemon).
- **Keep to the task.** If you find something else wrong, mention it in your
  report; don't fix it.
- **Keep it simple** (`workflow/base/rules/kiss.md`). Build what the task names, to
  the precision it needs. One green `just check` is enough: no repeated full
  runs, test loops or extra benchmarks unless the task asks.
- **Times to the human are US Eastern** (`workflow/base/rules/human-timezone.md`);
  written bare ("7:00 AM"), with a zone only when it isn't Eastern.
  Records stay in UTC.
- **YAGNI, and the cost of not doing it** (`workflow/base/rules/yagni.md`,
  `workflow/base/rules/cost-of-not-doing.md`). Build for today's need, not a foreseen
  one. Before any task, step or check, ask what the worst is if you don't do
  it; if it's not much, don't.
- **Tests**: add or update tests for what you change. Use the fake claude
  (`crates/bridle-claude/tests/fake-claude.py`); never run real `claude`, and
  never run `just test-live` or `just test-contract`.
- **Docs**: if you change behaviour described in `docs/design/`, update the
  doc in the same commit.
- **Before you finish, bring your branch up to date**: `git merge main` (the
  **local** `main`; never `origin/main` or any other remote ref, which is
  stale), resolve any conflicts, and re-run the checks. Your manager merges your
  branch into `main` only if it already contains `main`.
- **Done means `just check` passes.** Then commit on your branch with a clear
  message. You are asked to commit, on your branch only.
- **Report** to whoever gave you the task (the sender in its message header):
  `bridle send <sender> "done: <one-line summary>; <commit sha>"`. If you're
  blocked, ask: `bridle send <sender> --question "<question>"`, then wait for
  the answer.

- **On a message starting "Usage pause:"**: commit your work in progress,
  send whoever's waiting on you one line on where you are, and end your turn
  without starting anything new.

## Background processes

Any background process you start must satisfy all three of these requirements, or it can outlive your agent and degrade the machine for everyone else.

- **Bounded**: Use limited parallelism (e.g., `--test-threads 4`) rather than the full default. Never start open-ended or unbounded loops (e.g., 40 repetitions of a full test run in a loop). A worker that starts an unbounded background loop can push the host's load average high enough to affect every other agent on the machine.
- **Cleaned up before your turn ends**: Don't leave background jobs running past the turn that started them, except for a specific, tracked reason (e.g., you're genuinely waiting on a long `just check` run and will check on it next). Stop any background process before you report done.
- **Never disowned**: Don't use patterns like `nohup`, `disown`, or detached `setsid` that intentionally let a process outlive the agent's own process tree. Bridle's cleanup on stop only works for processes still attached to your agent; disowned processes escape that containment.

## Never

- Push, fetch, or merge from a remote (`origin/*`), merge your branch into
  anything, or switch to another branch.
  Merging `main` into your own branch is the one merge you do.
- Change files outside your worktree.
- Commit with `just check` failing, or skip hooks.
