# Changelog

All notable changes to bridle are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `bridle prime orchestrator` works for a project other than bridle (br-5c3a): the orchestrator role file is generic (`{project}` becomes the project's name, e.g. in the credentials-entry snippet) and an optional `<repo>/.bridle/roles/orchestrator.md` is appended; the state file is optional. Bridle's own specifics moved to `.bridle/roles/orchestrator.md`. The advisor role is generic with an optional `.bridle/roles/advisor.md`.
- Calmer orchestrator wake loop (v9t9): `wait-for-wake` polls 25 minutes (was 5), and the role file says to restart it first on every wake; `[orchestrator] waiter_grace` defaults to 15m (was 2m); the startup and handover steps drop the separate heartbeat; `bridle status` shows when the last wake was delivered and whether a waiter is open (`Status.waiter_open`, `last_wake_at`).
- Workflow path per machine (br-4fd6): `workflow` expands `~` and `$VAR`, `~/.bridle/config.toml` may override it (machine beats project), and a missing or unreadable workflow directory is an error at `serve`/`sync`/`prime`/`rules` and a failing `bridle doctor` check instead of silently dropping the base rules and skills.
- Automatic upgrade (q7rx 3, br-38c8): `[daemon] self_upgrade = true` (default off; on in bridle's own config) makes the daemon, on the CI watcher's minute tick and only when no agent is mid-turn, run the `restart --upgrade` path when a newer green commit is on main. A failed commit isn't retried until main moves on.
- `bridle restart --upgrade` / `upgrade: true` on `POST /v1/restart` (q7rx 2, br-b21b): builds the newest commit on main whose GitHub CI is green (`cargo install --path crates/bridle` in a throwaway worktree, in the background), then restarts in place. Nothing newer than the last upgrade means it says so and does nothing; a failed build leaves the daemon running and wakes the orchestrator and messages the human.
- A message can be marked unread again: `POST /v1/messages/{id}/unread`, `bridle inbox unread <id>...`, and `u` in the TUI inbox (br-a03b, n94h).
- Restart in place: `bridle restart [--wait SECS]` / `POST /v1/restart` (orchestrator and human only) waits for every agent to be idle (gives up, restarting nothing, at the timeout), then flushes and pushes the state branch, stops agents through the shutdown path and execs the daemon's own binary with the same args (same PID and terminal). After the restart every agent that was running is resumed, workers too, and told to carry on; the orchestrator gets a wake before and after, the human only hears of failures (q7rx, br-a618).
- Multiple named advisors: `scripts/claude-advisor [name]` allows running several advisors at once, each with a unique session name (`advisor-<name>` or `advisor`); suffix with `BRIDLE_SESSION_SUFFIX` for machine naming. All advisors share one identity, token, and inbox; each must sign its messages with its name and add only its own files when committing. Orchestrator session names shortened similarly (`orch` or `orch-<suffix>`, replacing the host-based default from br-47ba) (sfb3, br-df68).
- Incidents built (br-70af): task kind `incident` (orchestrator/human plan, resolve, drop and reopen; kept out of the queue and `ready`), a `system` notice to every agent on promote and to agents that spawn, resume or renew while it's active (`messages.incident_task`, schema V18), undelivered notices dropped and delivered ones followed by a "resolved" note, `task list -k`, `task done --resolution`, and active incidents in `bridle status`.
- `bridle wait-for-wake` wakes the orchestrator when an incident task is created (br-2b1b).
- Incidents design revised: an incident is a task of kind `incident` (potential/active/resolved on the existing task states) plus a persistent, self-retracting broadcast notice; no incidents table or commands (nc7r, br-264b).
- Handover notes are also written to the state branch as `handovers/<id>.md` (backfilled at start), and `bridle rebuild` restores them with ids and seq kept; `bridle rebuild --from-origin`, and a first start with `[state] push` on (default, or explicit opt-out) and no local branch, fetch `origin/bridle/state` (fast-forward only, never overwriting a local branch) (we2r, br-011b).
- `[state] push` pushes `bridle/state` to `origin` after flushes that committed (at most once a minute, trailing, and once on shutdown; never forced; a non-fast-forward stops it); failures and last push show in `bridle status` (we2r, br-93ad, br-e70f). **Default on as of br-e70f** (human-approved 2026-09-29); projects opt out with `push = false`.

### Changed

- `scripts/claude-orchestrator` and `scripts/claude-advisor` session names now include the short hostname (lowercased) to differentiate them across machines in Claude mobile; e.g., `bridle-orch-nuc` and `bridle-advisor-nuc`. The hostname can be overridden with `BRIDLE_SESSION_SUFFIX` (sfb3, br-47ba).
- The TUI inbox lists every task's open question after the unread messages (as `bridle inbox` does), so an opened question stays until answered; `q` closes an opened message; opening a message no longer marks it read, `d` marks it done. `bridle inbox show` no longer marks read by default: `--no-mark-read` is replaced by `--mark-read` (n94h, br-6441).
- Bridle's own workers gate on the full `just check` again (`check_worker` binding removed from `.bridle/config.toml`; `just check-affected` stays as a recipe) (qgma, br-0e42).
- `bridle land` skips `[integration] check` when the integration branch is an ancestor of the task branch (a fast-forward of an unchanged base; the notes say so) and runs it when the base has moved. Worker and manager prompts send check output to `/tmp/<task>-check.log`, judge by exit status, and read the tail only on failure A passing check's nextest test count is recorded in `<workspace>/last-full-test-count`; a count of 0, or under half or over double the last one, fails the landing, and workers are told to sanity-check their own count against it (qgma, br-0e42).
- The orchestrator is no longer woken when `main` moves (`main_moved` wake removed): a merge arrives as the manager's message and a red main as the CI-failure wake. The role no longer tells it to `gh run watch` (br-9e71, pdmd).
- `[orchestrator]` `note_tokens`, `plan_tokens`, `handover_tokens` accept abbreviated forms: `"150k"` or `"1.5M"` (case-insensitive; `k` = 1,000, `M` = 1,000,000) alongside plain numbers. Defaults changed from 150K / 210K / 255K to **150K / 180K / 200K** (by23, br-f05d).
### Fixed

- `bridle status` no longer calls a stopped agent's empty branch merged, or lists agents stopped by a daemon shutdown (they're due a resume); `bridle rm --delete-branch` uses the same test (br-f919, z4hd).
- `bridle stop-daemon` prints progress as it goes (requested, acknowledged with the agent count and the daemon's cap, shutdown complete with elapsed time) instead of one misleading line at the end, and its 60 s timeout error points at `bridle daemons` and the daemon log (q23k). `POST /v1/shutdown` now replies with `{"stop_limit_secs"}`.

### Added

- Orchestrator context tracking (br-1fdb, ct8m step 6): the supervisor emits `orchestrator.context` events (session id, tokens, window size, uptime) on the first reading, on a lower reading (when the context is compacted), and at most once per 10 minutes when the reading changes; supports `bridle events --kind orchestrator.context` queries to track sessions' starting context and peak growth, answering "how long can the orchestrator run" with actual data.
- Orchestrator supervision slice 2 (br-65b8): the daemon reads the orchestrator's context (the statusline file, the transcript as a fallback) and sends `context` wakes at `note_tokens`, `plan_tokens` and `handover_tokens`, each once per session and again after a `/compact`; `max_uptime` asks for a handover too. `bridle handover done` (`POST /v1/orchestrator/handover`) or the `handover_deadline` stops the session (SIGTERM, SIGKILL after 15 s) and relaunches it without counting a crash. `scripts/context-check.sh` is gone.

### Fixed

- `bridle ask` now notifies someone: it sends a pointer message (kind `question`) to `--to` (default: the caller's spawner, or the human), and `bridle answer` sends one back to the asker. Docs and comments that called claims SQLite-only are corrected: they are mirrored to `claims.toml` and restored by `rebuild` (br-b966).

- `bridle send` and `task note` refuse an empty or whitespace-only body (CLI and API); the worker skill, role and stop-check tell workers to report to their manager, not `human` (br-6fd7, hx7t).
### Changed

- `scripts/claude-orchestrator` and `scripts/claude-advisor` start leaner (ct8m step 4, br-72da): `--strict-mcp-config` plus settings keys that drop bundled skills, workflows and the claude.ai connectors, and denies for tools these roles never use. AskUserQuestion, Agent, ToolSearch, Cron and Monitor stay. First turn 22.8K to 18.8K tokens in a scratch Haiku run (`docs/spikes/08-lean-context-findings.md`).

- `bridle land` lands one squash commit per task (`<task id>: <title>`, the summary as body, `Task:`/`Branch:` trailers) instead of a `--no-ff` merge (br-1d3d).

### Added

- Human to-dos: `bridle task new --for-human` creates a task claimed by the human (never leased away) and sends one inbox message; the human finishes it with `bridle task done <id>` with no `--commit`; the orchestrator role lists them at every start (ex9q, br-c83e).
- The orchestrator's handover note is a bridle record: `bridle handover write --file <path>|-` (human and `external:orchestrator`), `handover list` and `show <id>`, `/v1/handovers`, schema version 17. The newest note wins and `bridle prime orchestrator` prints it with its age; older notes are kept and pruned with the events at 30 days, always keeping the newest. The role file's handover step uses it instead of committing the state file (br-4573).
- Roles take a `tools` key, passed as `--tools`; the built-in worker and manager get lean defaults (worker: Bash, Read, Edit, Write, Glob, Grep; manager: no Edit/Write, plus Agent), cutting a spawned agent's first turn by about a third (br-9fca, ct8m step 2).

- Orchestrator supervision, first half (br-a424): with `[orchestrator] enabled = true` the daemon watches `$BRIDLE_HOME/orchestrator.pid` (pid plus start time) and, when the session is dead and the pane tagged `@bridle=orchestrator` shows a shell on two checks 5 s apart, types the launcher into it, with crash-loop backoff (`relaunch_backoff`, `stable_after`) and an incident message to the human when it gives up, finds no tagged pane, or finds something else running. `scripts/claude-orchestrator` writes the pid file and registers a SessionStart hook, `bridle orchestrator note-session`, instead of pinning `--session-id`.
- Orchestrator supervision, second half (br-e949): the wake conditions of `scripts/orchestrator-watch.sh` (an agent exiting, crashing or stalling, a question to the human, a message to the orchestrator, all agents idle 15 min, usage at 93%/85%, a budget hold, a failed CI run on the integration branch, the branch moving) now live in the daemon, each firing once and queued while nobody waits. The orchestrator waits on `bridle wait-for-wake` (`GET /v1/orchestrator/wake`, a 5-minute long poll, `external:orchestrator` only); `[orchestrator] waiter_grace` (default 2m) makes a live session with no waiter an incident. The script and its `~/.bridle-orchestrator-*` state files are gone.
- `bridle stop-check` runs the project's `check_worker` command for a finished-looking worker whose HEAD has no recorded pass, records the pass in the worktree's git dir, and blocks with the output tail on failure (br-f671).
- `bridle stop-check` also blocks a worker whose tree looks finished (clean, commits ahead) but whose claimed task has no summary or `done:` report, telling it to run the commands rather than print them (br-d99e).
- `bridle init [--name N] [--integration BRANCH] [--stack python|typescript|rust]`: scaffolds `.bridle/config.toml` (integration branch from HEAD, `workflow`/`packs`, check command detected from `justfile`/`Cargo.toml`/`package.json`/`pyproject.toml`, worktree stubs) and the `.gitignore` entries; never overwrites, lists existing files as skipped (br-e0f4).
- `bridle doctor [--repo PATH]`: checks a project's setup (git repo, integration branch, config and the files it references, role prompts, `.gitignore`, `[ports]`, git >= 2.38, `claude`, `gh` when `[ci]` is on), prints ok/warn/FAIL with a one-line fix each, exits 1 on a failure (br-5a5d).
- landing an arch-revision task opens one re-evaluate task per capability with suspect requirements (listing the ids, to confirm or edit) and notifies the manager (br-beab).
- the landing notice to running workers says `spec changed under you: <ids/files>` when the landed commit touches a claimed task's declared impact (spec ids changed in design/specs, file globs); other workers keep the generic notice (br-66e2).
- `bridle spec export --scenario ID` (repeatable; `s-` or `r-` ids) and `--task ID` (the scenarios in a task's declared impact; exits 1 if none declared) narrow the json/gherkin export to selected scenarios (br-b85c).
- `bridle spec coverage [--root DIR] [--tests DIR ...] [--require-all] [--json]` lists executable scenarios whose id does not appear in test sources; scans text files under `--tests` directories (default `tests` and `test` if present) for scenario ids; exits 1 with `--require-all` if any unbound (br-b1e2).
- `bridle spec id [paths...] [--root DIR] [--ledger FILE] [--dry-run]`: writes stable ids (`{#r-xxxx}`, `{#s-xxxx}`) into spec headings that lack one, editing only those lines, unique across the spec set and never reused, via a committed `design/specs/.ids` ledger (br-41e1).
- `bridle spec export --format gherkin|json [--out DIR] [paths...]`: exports capability specs for test runners (gherkin: one `.feature` per capability, executable scenarios only, tagged with their `@tags` and scenario id; json: the whole AST with ids), refusing when the specs have errors; gherkin defaults to the gitignored `.bridle/cache/features/` (br-3058).
- `bridle spec check [paths...] [--root DIR] [--require-ids] [--json]`: validates capability spec files with the `bridle-spec` parser and prints `file:line:col: message` diagnostics; a requirement without an id is a warning (an error with `--require-ids`), any error exits non-zero (br-e531).
- `bridle land <task> [--branch B] [--check-cmd CMD]`, the integrator: merges the branch in `<workspace>/integration`, runs `[integration] check`, moves the integration branch with a guarded `update-ref`, then marks the task done; refuses conflicts, failed checks, a moved main, and architecture changes outside an `arch-revision`; emits `integrate.started/finished` (br-6dd6).
- `bridle arch-guard`, a PreToolUse hook (shipped in `workflow/base/hooks/`, rendered by `bridle sync`) denying worker edits under `design/architecture/` unless the worker has claimed an `arch-revision` task (br-7f7e).
- `bridle arch propose --title T --argument TEXT|-` creates an `arch-revision` task with the proposal; validates the architecture directory exists (br-357f).
- `bridle goals propose <goal-id> --change KEY=VALUE --why TEXT` (repeatable `--change`) creates a task proposing a change to the goal's firmness, priority, or stance; validates the goal exists (br-357f).
- `bridle probe <task-or-agent>|--branch B` runs `git merge-tree` against the integration branch and reports clean or the conflicting paths; `impact check` lists non-clean probes of claimed tasks' branches (conflict vs integration, warn between tasks); needs git 2.38 (br-2612).
- `bridle impact set|show` declares and prints a task's impact (spec ids and file globs), stored on the task record and rebuilt by `bridle rebuild` (br-9821).
- `bridle impact check [--json]` reports overlaps between in-flight tasks' declared impact (conflict, warn, info; exit 1 on a conflict) (br-3584).
- `bridle conflict list|resolve`: `impact check` opens a conflict (`C<n>`) for each shared scenario, once, and tells both claimants (or the managers, for an unclaimed task); resolve with `--compatible`, `--order A,B` or `--merge-into` (br-6774).
- `bridle trace down|up|orphans` over goals, architecture and specs; `serves=` on architecture elements, a 4-hex text hash for `traces=id@hash`, `bridle trace suspect` lists links whose recorded hash is stale (exit 1 if any), and `bridle trace confirm <id>` rewrites them to current (br-d226, br-dd44).
- `bridle send <agent> --task <id>` and `bridle task note <id> --notify <agent>`: the text goes on the task's thread and the recipient gets a short message naming the task; role prompts and skills use it for briefs, done reports and findings (ticket n8tj, br-9474).
- `bridle task search <words...>`: search for tasks by words in title, body, or summary (case-insensitive substring match, all words must match); includes done and dropped tasks; returns the same columns as `task list` (ticket br-f86f).
- `bridle task summary <id> --text|--file`: stores task summary on the task record and its state-branch file; `task show` prints it; `task done` warns when there is no summary (ticket tr7k).
- `bridle task new|edit|note` now accept `--body-file` and `--text-file` options (mutually exclusive with `--body` and positional `TEXT` respectively), allowing long task bodies and notes to be passed via file or stdin to avoid shell metacharacter permission denials (ticket br-3822).
- `bridle task done --branch` now removes the branch's agents, worktree and branch (refusing unless `--commit` is on the integration branch) and notes it on the task; `bridle status` lists stopped agents whose branch has merged (br-7d81).
- Port registry: `bridle port alloc [--pid N] [--label L]|release <port>|list`, `[ports] range`/`reserved` in config; the daemon frees a port when its owner agent exits or its pid dies (br-57be).
- `[worktrees] layout = "paired"` with `[worktrees.pair.<name>] path, mode = "worktree"|"symlink"` creates sibling repos' worktrees (or symlinks) beside the project's at `<root>/<name>`; setup runs in each, rm cleans all and refuses on a dirty member, the system prompt lists sibling paths (br-cc25).
- `[worktrees] layout = "root"` with `root = "/path/{task}"` (`{task}`, `{agent}`, `{project}`) puts new worktrees at a configured absolute path; `default` is unchanged; invalid roots are refused at config load (br-930e).
- `[worktrees] copy`: repo-relative files (e.g. gitignored `.env`, `.mcp.json`) copied from the project clone into each new worker worktree before setup runs, keeping their mode; a missing file is skipped with a warning; absolute or `..` paths are rejected (ticket br-3309).
- `[worktrees] setup` (with `setup_timeout_secs`, default 600): a shell command run in each new worker worktree, e.g. an install step; a failure or timeout fails the spawn and removes the worktree (ticket br-42dd).
- Cheaper builds: new worker worktrees start with a copy-on-write clone of the clone's `target/` on macOS (`[worktrees] warm_target`, default on), and workers' own gate is `{{commands.check_worker}}` (`commands.check_worker`, default `commands.check`; bridle's own project runs `just check-affected`).
- `~/.bridle/credentials.toml` (0600, a table per external principal, a key per project) replaces the per-principal token files: `BRIDLE_AS=<principal>` makes every command use that principal's token for the project it talks to (after `--token` and `$BRIDLE_TOKEN`), `bridle token create` saves the token there instead of printing it (when the project is known) and `token revoke` removes it, and a file looser than 0600 is refused; `scripts/claude-orchestrator` and `scripts/claude-advisor` set `BRIDLE_AS` (ticket t6kq, br-f4d1).
- `bridle launchd install|uninstall` (macOS): writes or removes a per-project LaunchAgent plist that runs `bridle serve`, and prints the `launchctl` commands without running them, so the daemon and its builds have no GUI responsible app and stop flashing Gatekeeper's Verifying window (ticket qr8z, br-936d).
- `bridle spawn --allow-tool <tool>` gives one spawn extra Claude Code tools, and `--env KEY=VALUE` gives it environment variables (e.g. a paid API key) that are never logged; both are kept on the agent and reapplied on resume and renew (br-4774).
- Disk usage monitor: every `[disk] check_interval` (default 1h, `0s` = off) the daemon logs and records as a `disk.checked` event the volume's free space and the sizes of the clone's `target/`, `wt/` and `.bridle/`, and messages the human once when free space falls under `[disk] min_free_gb` (default 20) (ticket m3wq).
- TUI inbox: `Enter` opens the selected message in full (marking it read), `r` replies from there, `Esc`/`Enter` dismisses; `bridle inbox show <id>` prints one message in full with header and body plus the reply command (marks it read by default, with `--no-mark-read` to skip); `bridle inbox read <id>...` marks one or more messages read (ticket fgu6).
- Vim workflow pack (`workflow/packs/vim/`, opt in with `packs = ["vim"]`): vader.vim testing convention, the `g:test_dir` temp-dir pattern, no reliance on `after/ftplugin/`, and a check-command rule that defers to the `commands.check` binding.
- Typescript pack enhancements: vitest adapter `workflow/packs/typescript/adapters/vitest-bridle/` with `registerBridleSpecs({ steps })` registers executable scenarios from `bridle spec export --format json` as vitest tests (br-a54d); typescript workflow pack itself (opt in with `packs = ["typescript"]`) for npm workspaces, vitest conventions, tsc type checking with no silent suppressions, dev-server hygiene, and a check-command rule that defers to the project binding.
- Python pack pytest plugin `workflow/packs/python/adapters/bridle_specs.py`: registers pytest-bdd scenarios from `bridle spec export --format json` (ids in test names, tags as markers, examples parametrized, `--bridle-spec`/`--bridle-scenario` selection), replacing `spec-to-feature.py` + `run-specs.py` (br-3b72).
- Workflow layers (P2): the shared base layer lives in `workflow/base/` in this repo, and `bridle rules explain` / `bridle rules diff` show how the layers resolve.
- Base workflow rules: six more rules (`doc-links`, `work-flow`, `ask-blocking`, `record-decisions`, `plan-discipline`, `out-of-scope`), harvested from data-contracts' working practice; the base `worker`/`manager`/`product-manager` role prompts and the manager skill are project-neutral, using `{{commands.check}}` and `{{branches.integration}}` (role prompt files now get `{{commands.check}}` substituted too).
- Components part 1: `[components.<id>]` in `.bridle/config.toml` (`paths`, `parent`, `docs`, `consumers`; unknown parents and cycles are config errors), an L4 component rule layer per chain, and `bridle rules explain|diff --component <id>`.
- Components part 2: tasks and spawns carry an optional `components` list (`task new|edit`, `spawn --component <id>`, repeatable; unknown ids rejected, never required); `task list --component X` matches `X` and its descendants; the daemon stores the list on the agent and sets `BRIDLE_COMPONENTS`; task files gain an optional `components` frontmatter array.
- Components part 3: `bridle prime worker|planner` prints the role's rules, facts and guide pointers, then, for each component in `--component` (else `BRIDLE_COMPONENTS`), its chain's rules/facts/guides under its own heading, docs pointers (README.md inline when ≤40 lines), and a one-line list of the components not named; nothing is rendered to files; `prime orchestrator` is unchanged.
- CI watcher: with `[ci] github = true`, the daemon polls GitHub Actions (via `gh`) for each new tip of the integration branch, emits `ci.completed`, messages the manager on a failure with the failed jobs, and `bridle status` shows the last result.
- `[[budget.schedule]]` period may omit `days`/`start`/`end` to be a named preset used only via `bridle budget override <name>`, and may set `max_workers`, applied through the live max-workers override while the override is in force and reverted when it ends or is cleared; `bridle budget` shows a period's `max_workers`; `GET /v1/budget` schedule `span` is now optional and gains `max_workers`.
- Terminal task state `integrated`, entered by `bridle task done <id> --commit <sha>` (the sha is recorded in the thread): it resolves the task's `blocks` edges, drops it from `bridle queue` and `ready`, and `reopen` works from it (ticket br-789a).
- Task size estimates: `S`, `M` or `L`, set with `bridle task new|edit --size` and shown in `task show`, `task list`, `queue` and `ready`, so small tasks can be picked when budget runs short; `bridle task edit --size none` clears a task's size (ticket br-0685, br-b30d).
- `.gitattributes` sets CHANGELOG.md to use the union merge driver, so branches that append to the changelog can be merged without conflicts (ticket br-e7f4).
- `bridle statusline` now shows context tokens (e.g., `40.0k`, `1.2M`) alongside the context percent, so the human can see the raw count; when tokens aren't available after a compact, only the percent is shown; `bridle statusline` now writes the session's context size to `~/.bridle/context/<session id>` and `scripts/claude-orchestrator` pins and records its session id (ticket c9zm).
- Daemon logs a shutdown request at WARN, naming the caller for `POST /v1/shutdown` or the signal (SIGINT/SIGTERM); open event streams already end on shutdown and the HTTP drain is bounded at 5s (tickets sed3, zm95, br-36fa).
- Newly filed task sends the running manager a `system` note ("task <id> filed: <title>; open tasks: N. Plan it or queue it."), at most once a minute; the manager role prompt says to run `bridle queue` and `bridle task list --state open` when idle or woken (br-3bb4).
- A role with no `system_prompt` now defaults to `<workflow>/base/roles/<role>.md` when `workflow` is set and the file exists; an explicit `system_prompt` still wins (ticket rl2v, br-f636).
- Built-in `manager` role now defaults to `autostart = true`: a project with no role config gets a manager at daemon start (`autostart = false` in `[roles.manager]` turns it off); bridle's own `product-manager` now sets `autostart = true` too (ticket qun8).
- Orchestrator's watcher wakes it with `CONTEXT <tokens>` when its own session passes 140K (`CONTEXT_WAKE`), once per crossing (ticket c9zm).
- Read-only local access without a token: a plain Claude Code session can run read commands (`bridle task list`, `bridle agents`, `bridle logs` ...) with no token; writes still need one.
- `bridle status` and `bridle usage` hide unnamed rate-limit windows.
- Test daemons no longer read the machine-wide `~/.bridle/config.toml`.

### Changed

- Tasks land as one squash commit each (`git merge --squash`, subject `<task id>: <task title>`, the worker's summary as body, `Task:` and `Branch:` trailers); `bridle rm --delete-branch` now accepts a squash-landed branch, recognised by its `Branch:` trailer (ticket sq4m).
- When a task lands (`bridle task done`), the daemon tells the other running workers that main moved, with the changed files, so they rebase or merge before their next commit; one message per agent per minute (br-07d6).
- A reply from a principal in `[messages] answer_for_human` (default `external:orchestrator`) to a message addressed to the human now closes it: it leaves the unread count and `bridle inbox` and the TUI show "answered by <who>: <first line>"; `Message` gains `answered_by`, `answered_reply` and `answered_line` (ticket h5qd, br-b29c).
- `bridle budget` now shows local-machine times, the applied `five_hour` thresholds with their source (override/schedule period/default), the current period's span, the next schedule change, why the state is what it is, each reading's age (stale by `max_staleness`) and non-`allowed` statuses; `bridle budget --schedule` prints the whole resolved schedule; `GET /v1/budget` gains `five_hour`, `schedule`, `reasons` and `age_secs` (br-75).
- Claude's `allowed_warning` rate-limit status no longer forces wind-down; it is shown by `bridle budget` but the configured thresholds and overrides alone decide the governor state; `rejected` still forces paused (ticket kv7d).
- `just test` (and so `just check`) runs git with an empty global config, like CI, so local runs no longer depend on your `init.defaultBranch` (g3ck, br-d0c3).
- `max_workers` is now enforced at worker spawn (409 at the cap), and `bridle budget max-workers <n>` (`--clear`) changes it live without a restart; after a usage pause, managers, the PM and the orchestrator always resume; the cap limits workers only.
- The human's inbox is for questions, blockers and decisions only; managers no longer send routine status notes; changes are recorded in the changelog.
- Per-project branch pattern: `[branches]` in `.bridle/config.toml` sets the `integration` branch workers branch from and managers merge into (default `main`), and an optional `release` branch no agent but the orchestrator may push; a project trial points `integration` at `bridle-adopt`; `state` is now a reserved agent name.

### Fixed

- `bridle budget --help` (and any `bridle budget` parse) no longer panics on a clap debug assert for a non-existent `conflicts_with = "action"` attribute; added tests for `--help` on budget and related subcommands to catch this class of bug.
- Help text for `bridle statusline`, `bridle prime`, `bridle task` and `bridle usage --by` now accurately reflect their supported roles and options.
- `bridle land` no longer leaves a worktree that has the integration branch checked out (the clone) showing the new tip as staged changes: it fast-forwards there (`merge --ff-only`) instead of `update-ref`, and refuses if that worktree has uncommitted changes (br-land).
- After a budget pause the governor resumes the manager and PM along with workers; `max_workers` limits workers only (ticket k7nr, br-0a50; code landed with y2eb).
- A resumed agent whose Claude Code session is gone no longer dies on its first turn on every resume; the daemon retries once on a fresh session in the same worktree, and `agent.exited` carries the claude `stderr_tail` (ticket p4ks, br-3ec1).
- `bridle renew` no longer waits on a budget hold (it replaces a session rather than adding load), so a hold can't leave the agent stopped; `--ignore-budget` is accepted and ignored on renew (ticket r3nh, br-1392).
- `bridle sync` no longer creates an empty `.claude/settings.json` when there are no hooks to write (br-082f).
- Dropping a claimed task now releases its claim, the claim lease check never moves a task that isn't `claimed`, and the daemon discards claims on non-`claimed` tasks when it loads, so a dropped task no longer lingers under "Claimed" in `bridle queue` (ticket b5br).
- Lists of tasks, edges, claims, open questions, agents and tokens now break `created_at` ties by insertion order, so rows created in the same millisecond no longer come back in either order; fixes a flaky components test (ticket f1ky).
- On Intel Macs (x86_64), `.cargo/config.toml` now ad-hoc code-signs binaries at link time, preventing crashes in macOS's system policy daemon when executing unsigned Mach-O binaries; the flag is a no-op on arm64 (ticket cs7x).
- TUI agents panel and inbox now scroll to keep the selected row visible (tickets yurx, 8ups, br-9e67).
- TUI agents and inbox lists no longer lose the highlight on the selected row (regression from making them scroll) (br-ee7d).
- TUI now shows agents spawned after it started: an event for an agent it has no row for triggers a refetch of the agents list, keeping the selected agent selected (ticket n4vk, br-ee7d).
- `bridle renew` under a budget hold refused only after stopping the agent, leaving it stopped; the hold check now comes first, so a refused renew changes nothing (ticket r3nh).
- An agent renewed and then resumed after a daemon restart before its new session's first turn died on its first turn (`--resume` of a session claude never wrote); `resume` now starts a fresh session in that case (`agents.session_started`, schema v13), and an abnormal claude exit is logged at warn with its stderr tail.
- Autostart now skips a role that already has an agent of that role (any state), not only one named after the role, so a restart no longer spawns a second manager beside e.g. `manager-2` (br-ee7d).
- Daemon shutdown no longer hangs while a client (`bridle tui`, `events --follow`) holds the event stream open; the stream now ends when shutdown begins, and the HTTP drain is bounded at 5s.

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
