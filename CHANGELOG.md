# Changelog

All notable changes to bridle are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- New `bridle spec export --format gherkin|json [--out DIR] [paths...]`: exports capability specs for test runners (gherkin: one `.feature` per capability, executable scenarios only, tagged with their `@tags` and scenario id; json: the whole AST with ids), refusing when the specs have errors; gherkin defaults to the gitignored `.bridle/cache/features/` (br-3058).
- New `bridle spec check [paths...] [--root DIR] [--require-ids] [--json]`: validates capability spec files with the `bridle-spec` parser and prints `file:line:col: message` diagnostics; a requirement without an id is a warning (an error with `--require-ids`), any error exits non-zero (br-e531).
- One `~/.bridle/credentials.toml` (0600, a table per external principal, a key per project) replaces the per-principal token files: `BRIDLE_AS=<principal>` makes every command use that principal's token for the project it talks to (after `--token` and `$BRIDLE_TOKEN`), `bridle token create` saves the token there instead of printing it (when the project is known) and `token revoke` removes it, and a file looser than 0600 is refused; `scripts/claude-orchestrator` and `scripts/claude-advisor` set `BRIDLE_AS` (ticket t6kq, br-f4d1).
- New `bridle launchd install|uninstall` (macOS): writes or removes a per-project LaunchAgent plist that runs `bridle serve`, and prints the `launchctl` commands without running them, so the daemon and its builds have no GUI responsible app and stop flashing Gatekeeper's Verifying window (ticket qr8z, br-936d).
- The orchestrator's watcher wakes it with `CONTEXT <tokens>` when its own session passes 140K (`CONTEXT_WAKE`), once per crossing; `bridle statusline` now writes the session's context size to `~/.bridle/context/<session id>` and `scripts/claude-orchestrator` pins and records its session id (ticket c9zm).
- New `bridle send <agent> --task <id>` and `bridle task note <id> --notify <agent>`: the text goes on the task's thread and the recipient gets a short message naming the task; role prompts and skills use it for briefs, done reports and findings (ticket n8tj, br-9474).
- Fixed: dropping a claimed task now releases its claim, the claim lease check never moves a task that isn't `claimed`, and the daemon discards claims on non-`claimed` tasks when it loads, so a dropped task no longer lingers under "Claimed" in `bridle queue` (ticket b5br).
- Tasks land as one squash commit each (`git merge --squash`, subject `<task id>: <task title>`, the worker's summary as body, `Task:` and `Branch:` trailers); `bridle rm --delete-branch` now accepts a squash-landed branch, recognised by its `Branch:` trailer (ticket sq4m).
- Tasks record their landing: `bridle task done --branch` (with `--commit`) and the new `bridle task summary <id> --text|--file` are stored on the task and its state-branch file, `task show` prints branch, commit and summary, and `task done` warns when there is no summary (ticket tr7k).
- New `bridle task search <words...>`: search for tasks by words in title, body, or summary (case-insensitive substring match, all words must match); includes done and dropped tasks; returns the same columns as `task list` (ticket br-f86f).
- Fixed: lists of tasks, edges, claims, open questions, agents and tokens now break `created_at` ties by insertion order, so rows created in the same millisecond no longer come back in either order; fixes a flaky components test (ticket f1ky).
- On Intel Macs (x86_64), `.cargo/config.toml` now ad-hoc code-signs binaries at link time, preventing crashes in macOS's system policy daemon when executing unsigned Mach-O binaries; the flag is a no-op on arm64 (ticket cs7x).
- Fixed: the TUI agents and inbox lists no longer lose the highlight on the selected row (regression from making them scroll).
- `.gitattributes` sets CHANGELOG.md to use the union merge driver, so branches that append to the changelog can be merged without conflicts (ticket br-e7f4).
- New `[worktrees] copy`: repo-relative files (e.g. gitignored `.env`, `.mcp.json`) copied from the project clone into each new worker worktree before setup runs, keeping their mode; a missing file is skipped with a warning; absolute or `..` paths are rejected (ticket br-3309).
- Tasks have an optional estimated size, `S`, `M` or `L`, set with `bridle task new|edit --size` and shown in `task show`, `task list`, `queue` and `ready`, so small tasks can be picked when budget runs short; `bridle task edit --size none` clears a task's size (ticket br-0685, br-b30d).
- New terminal task state `integrated`, entered by `bridle task done <id> --commit <sha>` (the sha is recorded in the thread): it resolves the task's `blocks` edges, drops it from `bridle queue` and `ready`, and `reopen` works from it (ticket br-789a).
- Changed: Claude's `allowed_warning` rate-limit status no longer forces wind-down; it is shown by `bridle budget` but the configured thresholds and overrides alone decide the governor state. `rejected` still forces paused (ticket kv7d).
- `bridle task new|edit|note` now accept `--body-file` and `--text-file` options (mutually exclusive with `--body` and positional `TEXT` respectively), allowing long task bodies and notes to be passed via file or stdin to avoid shell metacharacter permission denials (ticket br-3822).
- New disk usage monitor: every `[disk] check_interval` (default 1h, `0s` = off) the daemon logs and records as a `disk.checked` event the volume's free space and the sizes of the clone's `target/`, `wt/` and `.bridle/`, and messages the human once when free space falls under `[disk] min_free_gb` (default 20) (ticket m3wq).
- TUI inbox: `Enter` opens the selected message in full (marking it read), `r` replies from there, `Esc`/`Enter` dismisses (ticket fgu6).
- Fixed: the TUI now shows agents spawned after it started: an event for an agent it has no row for triggers a refetch of the agents list, keeping the selected agent selected (ticket n4vk).
- New `[worktrees] setup` (with `setup_timeout_secs`, default 600): a shell command run in each new worker worktree, e.g. an install step; a failure or timeout fails the spawn and removes the worktree (ticket br-42dd).
- Fixed: `bridle renew` under a budget hold refused only after stopping the agent, leaving it stopped; the hold check now comes first, so a refused renew changes nothing (ticket r3nh). Automatic context renewals already bypass the check.
- Fixed: an agent renewed and then resumed after a daemon restart before its new session's first turn died on its first turn (`--resume` of a session claude never wrote). `resume` now starts a fresh session in that case (`agents.session_started`, schema v13), and an abnormal claude exit is logged at warn with its stderr tail.
- Fixed: TUI inbox and agents panel now scroll: each table maintains TableState to track view offset as the selection moves, so selecting beyond the visible rows keeps the selection in view (tickets 8ups, yurx).
- A `[[budget.schedule]]` period may omit `days`/`start`/`end` to be a named preset used only via `bridle budget override <name>`, and may set `max_workers`, applied through the live max-workers override while the override is in force and reverted when it ends or is cleared. `bridle budget` shows a period's `max_workers`; `GET /v1/budget` schedule `span` is now optional and gains `max_workers`.
- New `vim` workflow pack (`workflow/packs/vim/`, opt in with `packs = ["vim"]`): vader.vim testing convention, the `g:test_dir` temp-dir pattern, no reliance on `after/ftplugin/`, and a check-command rule that defers to the `commands.check` binding.
- New `typescript` workflow pack (`workflow/packs/typescript/`, opt in with `packs = ["typescript"]`): npm workspaces, vitest conventions, tsc type checking with no silent suppressions, dev-server hygiene, and a check-command rule that defers to the project binding.
- `bridle budget` now shows local-machine times, the applied `five_hour` thresholds with their source (override/schedule period/default), the current period's span, the next schedule change, why the state is what it is, each reading's age (stale by `max_staleness`) and non-`allowed` statuses; `bridle budget --schedule` prints the whole resolved schedule. `GET /v1/budget` gains `five_hour`, `schedule`, `reasons` and `age_secs`.
- Fixed: autostart now skips a role that already has an agent of that role (any state), not only one named after the role, so a restart no longer spawns a second manager beside e.g. `manager-2`.
- Components part 3: `bridle prime worker|planner` prints the role's rules, facts and guide pointers, then, for each component in `--component` (else `BRIDLE_COMPONENTS`), its chain's rules/facts/guides under its own heading, docs pointers (README.md inline when ≤40 lines), and a one-line list of the components not named. Nothing is rendered to files; `prime orchestrator` is unchanged.
- The built-in `manager` role now defaults to `autostart = true`: a project with no role config gets a
  manager at daemon start (`autostart = false` in `[roles.manager]` turns it off).
- Fixed: daemon shutdown no longer hangs while a client (`bridle tui`, `events --follow`) holds the event stream open; the stream now ends when shutdown begins, and the HTTP drain is bounded at 5s.

- Cheaper builds: new worker worktrees start with a copy-on-write clone of the clone's `target/`
  on macOS (`[worktrees] warm_target`, default on), and workers' own gate is
  `{{commands.check_worker}}` (`commands.check_worker`, default `commands.check`; bridle's own
  project runs `just check-affected`).
- Components part 2: tasks and spawns carry an optional `components` list (`task new|edit`,
  `spawn --component <id>`, repeatable; unknown ids rejected, never required). `task list
  --component X` matches `X` and its descendants; the daemon stores the list on the agent and
  sets `BRIDLE_COMPONENTS`. Task files gain an optional `components` frontmatter array.
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
