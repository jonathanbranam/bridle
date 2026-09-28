# Changelog

All notable changes to bridle are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- Cheaper builds: new worker worktrees start with a copy-on-write clone of the clone's `target/`
  on macOS (`[worktrees] warm_target`, default on), and workers' own gate is
  `{{commands.check_worker}}` (`commands.check_worker`, default `commands.check`; bridle's own
  project runs `just check-affected`).
- Components part 1: `[components.<id>]` in `.bridle/config.toml` (`paths`, `parent`, `docs`,
  `consumers`; unknown parents and cycles are config errors), an L4 component rule layer per
  chain, and `bridle rules explain|diff --component <id>`.
- CI watcher: with `[ci] github = true`, the daemon polls GitHub Actions (via `gh`) for each new
  tip of the integration branch, emits `ci.completed`, messages the manager on a failure with
  the failed jobs, and `bridle status` shows the last result.
- The base `worker`/`manager`/`product-manager` role prompts and the manager skill are
  project-neutral: they use `{{commands.check}}` and `{{branches.integration}}` (role prompt
  files now get `{{commands.check}}` substituted too), and `bridle sync`'s CLAUDE.md block points
  every role at the rule files instead of the orchestrator-only `bridle prime`.
- `bridle statusline` now shows context tokens (e.g., `40.0k`, `1.2M`) alongside the context
  percent, so the human can see the raw count; when tokens aren't available after a compact,
  only the percent is shown.
- `bridle inbox show <id>` prints one message in full, with header and body, plus the reply
  command; marks it read by default, with `--no-mark-read` to skip. `bridle inbox read <id>...`
  marks one or more messages read (the same endpoint `--mark-read` on list uses, one at a time).
- Six more base workflow rules (`doc-links`, `work-flow`, `ask-blocking`, `record-decisions`,
  `plan-discipline`, `out-of-scope`), harvested from data-contracts' working practice.
- `max_workers` is now enforced at worker spawn (409 at the cap), and `bridle budget
  max-workers <n>` (`--clear`) changes it live without a restart. After a usage pause,
  managers, the PM and the orchestrator always resume; the cap limits workers only.
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
- Per-project branch pattern: `[branches]` in `.bridle/config.toml` sets the
  `integration` branch workers branch from and managers merge into (default `main`),
  and an optional `release` branch no agent but the orchestrator may push. A project
  trial points `integration` at `bridle-adopt`. `state` is now a reserved agent name.

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
