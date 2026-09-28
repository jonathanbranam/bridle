# Orchestrator state

The orchestrator's working notes for handing over between sessions. The role
itself is in `.bridle/roles/orchestrator.md`. Update this file whenever the
queue, open items or decisions change. Last updated 2026-09-28 07:20 UTC,
near the end of the fourth orchestrator session.

## First, for the incoming orchestrator

- **v0.2.0 is tagged** (2683d6d, 2026-09-28 ~06:25 UTC): P0 complete, two
  local runs 297/297, GitHub CI green on Linux and macOS. n6gy resolved again.
- **For the human in the morning:** rebuild and restart the daemon
  (`cargo install --path crates/bridle`, then restart). The running daemon is
  the 9608376 build, so htp6b auto-renew, 78sp, 4eep, x7gp and the shutdown fix
  aren't live yet. After the restart, resume any `lost` workers.
- **Also for the human: turn on the budget schedule** (n9qh part 1, cdb4ed0).
  It ships with no periods, and a project config may only lower thresholds,
  so the periods go in the machine-wide `~/.bridle/config.toml` (which doesn't
  exist yet). The human creates it, then restarts. The recommendation, in the
  machine's local time (the workday hours are a guess; the human confirms):

  ```toml
  [[budget.schedule]]
  name = "night"
  days = "all"
  start = "23:00"
  end = "07:00"
  hold_at = 90
  wind_down_at = 93
  stop_at = 95

  [[budget.schedule]]
  name = "workday"
  days = ["mon", "tue", "wed", "thu", "fri"]
  start = "09:00"
  end = "17:00"
  hold_at = 90
  wind_down_at = 93
  stop_at = 95
  ```

  n9qh part 2, the thermostat-style `bridle budget override`, is still to do.
- **Paging:** until the restart, `bridle logs` and `bridle events` return the
  oldest 500 lines without `--since` (x7gp is fixed on main). Find the latest seq by paging, and read an
  agent's current turn with `--since`.
- **Once the daemon restarts on a build with htp6b** (b7b2051 or later), context
  renewal is automatic; until then renew by hand with `bridle renew`.

## Who's running

- **`pm-1`** (product manager, Sonnet), renewed 05:53 UTC, 145K at 07:15.
  It owns the queue in `bridle task`.
- **`manager-2`** (development manager, Sonnet), renewed 07:17 UTC.
- **Workers:** `s8kn-resolve-tickets` (Haiku) and `r7cs-token-file-fix`
  (Sonnet: the statusline reads its token from a file, not `$BRIDLE_TOKEN`,
  and doesn't claim the token is read-only).
- The split is interim, by configuration (ticket tx3f).

## Queue

Done since v0.2.0 and verified on CI: w4tb (wall time), n9qh parts 1 and 2
(the schedule and `bridle budget override`), mt7r (`just check-affected`),
r7cs (df32928; its setup docs are being fixed). Local double-check of
aab6ee6/df32928 was pending at 07:15. Next: pm-1's picks from `bridle task`.

Parked: a7h3 (agreed with pm-1: the message-human workaround works), and
`bridle/mcp-1` (don't merge or delete it; u6wk).

## The human's decisions (2026-09-27 and 28)

- P0: task records on a state branch (c7eb); questions inline in the task
  thread with a SQLite index (c5a8). Migrate bridle's own work after P0,
  as-is (tskm).
- Rules for every role: KISS (`kiss.md`), YAGNI (`yagni.md`), and "what's the
  worst if we don't?" (`cost-of-not-doing.md`), all in `.bridle/rules/`.
- The merger pushes `main` after each merge; workers never touch `origin/*`.
  Merged branches are removed (`bridle rm <name> --delete-branch`). SemVer
  releases, cut by the orchestrator (`v0.1.0` tagged).
- Context: agents are ephemeral. Keep contexts well under 200K and renew in
  place (htp6). This includes the orchestrator.
- Split product and development managers (tx3f).
- Flaky tests: fix them properly, never with retries (f1ky). Hang guards
  are 60 s; the misdiagnosed promptness test was fixed.
- Status line: display only (real context %, 5h/7d, model, 📁 folder,
  🌿 branch). Recording was dropped (s8kn); counts come later (r7cs).
- Talk to the human in US Eastern time; record in UTC (`human-timezone.md`).
  The human is asleep until about 7:00 AM.
- The orchestrator owns d4mz. `scripts/claude-orchestrator` starts a new
  orchestrator with Remote Control on (as `bridle-orch`).
- n9qh (budget schedule and thermostat override) is after the cutover, not P0.
  But when the P0 track is idle and waiting on the human, pick up n9qh or
  other polish work to keep the workers busy (2026-09-28).

## Things to know

- **The daemon reads `.bridle/config.toml` and the role prompts only at
  startup**, and runs the installed binary. Rebuild with `cargo install
  --path crates/bridle` (the orchestrator can); only the human restarts the
  daemon. The binary was installed at `9608376` (P0-4, s8kn). The running
  daemon started at ~02:23 UTC, on `cce2bec`.
- **A daemon restart stops busy workers** after their 30 s grace. Resume them
  with `bridle resume <name>`, and tell each one the daemon restarted.
- **`bridle send` can't address `external:orchestrator`** (a7h3). Agents
  reach the orchestrator by messaging `human`; the watcher wakes on
  questions.
- **The watcher**: since P0-3b, `bridle inbox --json` is
  `{messages, open_questions}`, and the watcher handles that. Run exactly
  one. A backgrounded `&` in a Bash call orphans a copy that can't wake you.
- **Verifying under load**: `just check` twice, once the 5-minute load
  average is under 20. The two runs can straddle a merge; the second then
  covers it. After tonight's fixes, both of the last two runs were fully green.
- **The manager's git**: plain commands from the clone by branch name.
  `git -C <worktree>` is allowed only for `status`, and pipes are denied.

## Waiting on the human

- **sqt6, a short command alias** (pm-1's m-0616): `br` is taken (beads_rust).
  Recommendation: none for now; a shell alias of the human's own costs
  nothing. Low stakes, so no worker until they say.
- **j2vq, orchestrator permissions** (manager-2's m-0655, branch held
  unmerged): it adds `Bash(bridle *)` to the repo-wide `.claude/settings.json`,
  which grants it to every Claude Code session in the repo. Recommendation:
  don't merge it as-is; if wanted, pass it in `scripts/claude-orchestrator`
  (`--allowedTools`) so only the orchestrator gets it.
- **v4nk, FYI:** merged without escalation (bfd07c5). Workers are now refused
  agent lifecycle calls. It's a tightening that matches the role's design, so
  it was left in; the human can ask for a revert.

## Deferred

- **MCP server and permission prompts:** nice-to-haves.
- **`bridle take` / `give`:** spike 7r7m needs the human at a terminal.
- **P2 and later:** need P0 first.

## Done on 2026-09-28, fourth session (merged, verified green locally, pushed)

- P0-5 (`bridle rebuild`), P0-6a (the gate, signed off by the orchestrator)
  and P0-6b (68 tasks imported, idempotent; README states the post-cutover
  rule). P0 is complete.
- htp6b (automatic context renewal, plus two renew races and the message
  delivery fifo race), 78sp (built-in tools denied by default; `Agent`
  allowed again, da5fe47), and the Linux CI fixes listed above.
- Rules and tickets: `human-timezone.md` (bare Eastern times to the human),
  x7gp, n9qh, n6gy reopened.

## Done on 2026-09-28, third session (merged, verified green, pushed)

- htp6a (`bridle renew`), kc4v (accurate context via `get_context_usage`),
  P0-3a/b (questions on tasks: `ask`, `answer`, inbox), P0-4a/b (claims and
  leases), the f1ky hang-guard timeouts and the promptness-test fix, s8kn
  (status line).
- Tickets: kc4v, d4mz (step 1 done), mt7r, w4tb, s8kn, r7cs; f1ky updated.
- Rules `yagni.md` and `cost-of-not-doing.md`; the merge-cleanup step; 27
  merged branches deleted.

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
- **`bridle status` lists `nimbus_quill 0%`,** an unnamed `get_usage` entry
  that carries a utilization. It's harmless, but decide whether status should
  show only named windows.
