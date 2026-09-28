# Changelog

All notable changes to bridle are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- `bridle spawn --allow-tool <tool>` gives one spawn extra Claude Code tools, and
  `--env KEY=VALUE` gives it environment variables (e.g. a paid API key) that are
  never logged. Both are kept on the agent and reapplied on resume and renew.
- Workflow layers (P2): the shared base layer lives in `workflow/base/` in this repo,
  and `bridle rules explain` / `bridle rules diff` show how the layers resolve.
- Read-only local access without a token: a plain Claude Code session can run read
  commands (`bridle task list`, `bridle agents`, `bridle logs` ...) with no token;
  writes still need one.
- `bridle status` and `bridle usage` hide unnamed rate-limit windows.
- Test daemons no longer read the machine-wide `~/.bridle/config.toml`.
- The human's inbox is for questions, blockers and decisions only; managers no longer
  send routine status notes. Changes are recorded here.

## [0.3.0] - 2026-09-28

P1 complete.

- `bridle send role:<name>` fans out to every live agent with that role.
- `bridle prime orchestrator`: one-command orchestrator handover.
- `bridle stop-check`: a worker can't end its turn holding a claimed task with no
  handoff note (Claude Code's Stop hook).
- `bridle task note`, and task claim ownership (`claimed_by`) in the API and CLI.
- Worker principals are refused on agent lifecycle endpoints.
- `bridle send` and `bridle spawn` take `--text-file` / stdin.
- Budget schedule by time of day, and `bridle budget override`.
- `just check-affected`, a fast local test path.
- `bridle usage` reports each agent's busy and wall time.
- The status line shows bridle counts, read with its own token file.

## [0.2.0] - 2026-09-28

P0 complete.

- Task records on a state branch: `bridle task new|show|edit|list|drop|reopen`,
  `bridle dep`, `bridle ready`, `bridle rebuild`, and bridle's own tickets imported.
- Questions on tasks (`bridle ask`, `bridle answer`, inbox), claims and leases
  (`bridle claim`, `bridle release`).
- Context: each agent's context size is measured and shown, `bridle renew` renews
  an agent in place, and the context governor renews automatically.
- Claude Code's own messaging, subagent, scheduling and remote-trigger built-ins are
  denied by default (the Agent tool is allowed); spawned agents use project-only
  settings.
- `bridle logs` shows the latest lines by default; `bridle rm` refuses a worktree with
  open files unless `--force`.
- Fixes: a Linux-only shutdown race, held messages delivered when the governor
  recovers, and several flaky tests.

## [0.1.0] - 2026-09-27

- Initial release
- Daemon and CLI for spawning and supervising headless Claude Code agents
- Worktree management and agent lifecycle control
- Basic message routing and inbox support
- Configuration and containment (process groups, system limits)
