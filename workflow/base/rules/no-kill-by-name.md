---
id: no-kill-by-name
severity: must
roles: [orchestrator, project-manager, manager, worker, reviewer, advisor]
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

Not mechanically enforced yet.
