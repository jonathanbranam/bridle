# Orchestrator state

The orchestrator's working notes for handing over between sessions. The role
itself is in `.bridle/roles/orchestrator.md`. Update this file whenever the
queue, open items or decisions change. Last updated 2026-09-28 01:10 UTC,
after the second orchestrator session's daemon restart.

## Who's running

- **`pm-1`**, the product manager (`product-manager` role, Sonnet), spawned
  2026-09-28. It owns the backlog: triage, right-sized briefs, priority order.
  It sends prepared tasks to the development manager by message until P0-6
  moves the queue into `bridle task`.
- **`manager-2`**, the development manager (`manager` role, Sonnet). It runs
  prepared tasks two workers at a time, reviews, merges and pushes `main`.
  It replaced `manager-1`, which was stopped for heavy context.
- The split is interim, by configuration; ticket tx3f holds the full design.

## Queue (pm-1's, as briefed)

Track 1, P0: P0-3 questions on tasks, then P0-4 claims and leases (lease
renewed by agent activity), then P0-5 `rebuild`, then P0-6 (ticket tskm). P0-6
is the migration: a verification gate whose results I check, then all open
work imported as-is, with no triage and the files left in place.

Track 2: the context governor (ticket htp6). Measuring is built; next, spawn
into an existing worktree plus `bridle renew`, then `[context]` wind-down and
handoff. After it: 78sp, 4eep, a7h3. The f1ky flakes are low priority.

Parked: `bridle/mcp-1` (WIP `cae932c`); don't merge it. `v0.2.0` goes out
when P0 is complete.

## The human's decisions this session (2026-09-27)

- P0: task records on a state branch (c7eb); questions inline in the task
  thread with a SQLite index (c5a8).
- Migrate bridle's own work into bridle after P0, as-is (tskm).
- KISS (`.bridle/rules/kiss.md`): Haiku for light work; nice-to-haves only
  roughly right; the account-wide usage guard exact.
- The merger pushes `main` after each merge; workers never touch `origin/*`.
  SemVer releases, cut by the orchestrator on verified `main` (`v0.1.0`
  tagged).
- Context: agents are ephemeral. Keep contexts well under 200K, size tasks
  to fit, and renew agents in place (htp6). This includes the orchestrator.
- Split product and development managers (tx3f). The orchestrator may stop
  managers (`Bash(bridle stop manager-*)` in `.claude/settings.local.json`)
  and files its own tickets.

## Things to know

- **The daemon reads `.bridle/config.toml` only at startup**, and runs the
  installed binary. Config and code changes need `cargo install --path
  crates/bridle` (I can run it) and a daemon restart (only the human).
- **`bridle send` can't address `external:orchestrator`** (a7h3). Agents
  reach me by messaging `human`, with `--question` when they need an answer;
  the watcher wakes on questions and remembers the ones it has reported
  (`~/.bridle-orchestrator-seen-questions`).
- **The manager's git**: plain commands from the clone by branch name.
  `git -C <worktree>` is allowed only for `status`, and pipes are denied.
- **`bridle-claude` process_test** has 5 s timeouts that flake under load
  (f1ky). Re-run before calling `main` red.

## Waiting on the human

- **Optional:** set `bridle statusline` as the `statusLine` command in
  `~/.claude/settings.json`. Its parser is unverified against the real JSON
  (spike m9wt).

## Deferred

- **MCP server and permission prompts:** nice-to-haves.
- **`bridle take` / `give`:** spike 7r7m needs the human at a terminal.
- **P2 and later:** need P0 first.

## Done on 2026-09-27, second session (merged, verified green, pushed)

- `bridle cost audit`, `bridle usage --by role|model|agent`, the governor's
  per-role model step-down (and its precedence fix), held messages delivered
  when the governor recovers, P0-1 (task records, state branch), P0-2 (edges,
  `dep`, `ready`), context measuring, and a timestamp-precision fix.
- Tickets: tskm, f1ky, tx3f, 78sp, a7h3, 4eep, htp6. Resolved c7eb and c5a8.

## Done on 2026-09-27, first session (all merged into `main`, all verified green)

- **Fixes and small gaps:**
  - the v1 gaps: SSE reconnect, `events` defaults, `logs` tool-input
    rendering, `token list/revoke`, agent counts in `daemons`, spawn
    readiness, agent identity in the system prompt;
  - the e2e tests' env leak;
  - the shutdown panic, resuming the manager after a clean shutdown, the
    shutdown notice, and the test-daemon leak;
  - junk windows from `get_usage`;
  - the containment sweep's per-pid `ps` scans, which made stops take minutes;
  - three test races that only surfaced under load.
- **Budget governor**, core and wind-down (usage and budget design).
- **`bridle statusline`** and the ledger for sessions bridle doesn't host
  (P0a).
- **`bridle-tui`**: agents list, event tail, per-agent logs, inbox and
  reply.
- **Spikes:** u7pw (get_usage) and 03 / a7w8 (permission prompts need an MCP
  tool).
- **Tickets** for the run's findings: tk3m, q7fx, n6gy, w8bz, zk4p, j2vq,
  v4nk, r5hc, m9wt.
- **Merge policy** and the $50 spend caps.

## Findings not yet ticketed

- **Workers launch heavy, disowned background load.** `deflake` started two
  loops of 40 full test runs (load average 104). Disowned processes escape
  bridle's stop cleanup and slow everyone down. The worker role prompt should
  require background processes to be bounded, capped (`--test-threads 4`)
  and cleaned up.
- **`lifecycle_test interrupt_during_sleep_ends_the_turn_and_agent_stays_usable`**
  failed once under load. `deflake` couldn't reproduce it, and n6gy was moved
  to resolved anyway. Reopen it if the test fails again.
- **`bridle status` lists `nimbus_quill 0%`,** an unnamed `get_usage` entry
  that carries a utilization. It's harmless, but decide whether status should
  show only named windows.
