---
id: no-kill-by-name
severity: must
roles: [orchestrator, project-manager, manager, worker, reviewer, document-reviewer, advisor, aide, prototyper, designer]
locked: true
---
Never use `pkill -f`, `pgrep -f` piped to `kill` or `xargs kill`, `killall`,
or kill by a pattern match of any kind. Kill only a pid you started and own.

For processes you own:

- `$!` of a background command
- the shell job id (`kill %1`)
- `pkill -P $$` for your own immediate children

If a stray process seems stuck and you didn't start it, leave it and report it
on the task thread instead of trying to kill it.

Why: 2026-09-29, cause fx7x: workers running `pkill -f "just check"` killed the
live orchestrator twice because `pkill -f` matches any process whose argv
contains the pattern. The orchestrator's launcher argv contained the pattern.

Enforced by: `Bash(pkill *)` and `Bash(killall *)` in every role's deny list (and in
the sessions' settings), and the `bridle kill-guard` PreToolUse hook, which
refuses `pgrep ... | xargs kill`, `kill $(pgrep ...)` and `pkill` inside a compound command.
Plain `kill <pid>` is allowed. To stop a background task, use `TaskStop`.

Stopping a wait (`bridle agent wake`), arriving with part 2 of ticket 75h2 (not built yet):

- A new wait from the same session replaces the old one, so starting a fresh waiter is always
  safe and enough. "Same session" is matched by the session that started the wait, not by
  identity: identities are shared (every project's aide is `external:aide`; two unnamed advisors
  are both `external:advisor`), and a match by identity would have them end each other's waits.
- The replaced waiter prints a message saying it was superseded and exits with code 5. It marks
  nothing read.
- To stop a waiter without replacing it, run `bridle agent wake --stop`. It ends this session's
  open wait through the daemon, with no signals or pids.
