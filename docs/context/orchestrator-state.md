# Orchestrator state

The orchestrator's working notes for handing over between sessions. The role
itself is in `.bridle/roles/orchestrator.md`. Update this file whenever the
queue, open items or decisions change. Last updated 2026-09-28 03:15 UTC,
at the handover from the third orchestrator session.

## First, for the incoming orchestrator

- **Renew the heavy contexts.** Since the restart at ~02:23 UTC,
  `context_tokens` is accurate (kc4v), and at 03:10 UTC it read:
  `manager-2` 293K, `htp6b-context-governor` 321K, `pm-1` 191K. All are past
  the ~200K limit. `bridle renew <agent>` (htp6a) stops an agent and starts a
  fresh one in the same worktree, role and model. Renew each one between
  turns, when it's idle, and send the fresh one a short handover. For a
  worker, give it the task brief and "check git log/status on your branch
  and carry on". For manager-2, tell it what's in flight: which worker is on
  which task, the queue below, and that pm-1 feeds it. For pm-1, tell it the
  queue and to read `docs/questions/open/`. htp6b (the automatic wind-down)
  isn't merged yet, so this is manual for now.
- **Then keep both tracks moving overnight.** The human is asleep until about
  7:00 AM ET (11:00 UTC). Verify each merge, and bring back only what needs them.

## Who's running

- **`pm-1`**, the product manager (`product-manager` role, Sonnet). It owns
  the backlog: triage, right-sized briefs, priority order, sent to manager-2
  by message until P0-6 moves the queue into `bridle task`.
- **`manager-2`**, the development manager (`manager` role, Sonnet). It runs
  up to two workers, reviews, merges, pushes `main`, and now also runs
  `bridle rm <name> --delete-branch` after each merge.
- **Workers at handover:** `p0-5-rebuild` (track 1) and
  `htp6b-context-governor` (track 2, the `[context]` wind-down; it was sent
  back once for a grace-window fix).
- The split is interim, by configuration (ticket tx3f).

## Queue (pm-1's)

Track 1, P0: P0-3 and P0-4 are done. Next is P0-5 `bridle rebuild`
(running), then P0-6 (ticket tskm), the migration: a verification gate whose
results the orchestrator checks, then all open work imported as-is. When P0
is complete, tag `v0.2.0` on verified `main` (operating-model.md, Releases).

Track 2: htp6b wind-down (running), then 78sp, 4eep, a7h3.

Small items pm-1 holds for free slots, on Haiku: the `stop-daemon` message
("received; shutting down gracefully, may take up to 30s"), and human-friendly
numbers (152k, $11.6) in the TUI, `bridle usage` and `bridle agents`.

Backlog, not scheduled: d4mz step 2 (`bridle prime orchestrator`, P1), w4tb
(wall-time tracking), mt7r (run only the tests a change can affect), r7cs
(status-line counts with a read-only token).

Parked: `bridle/mcp-1` (WIP `cae932c`). Don't merge or delete it. It's
recorded in build-order.md and ticket u6wk.

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
  The human is asleep until about 7:00 AM ET.
- The orchestrator owns d4mz. `scripts/claude-orchestrator` starts a new
  orchestrator with Remote Control on (as `bridle-orch`).

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

Nothing.

## Deferred

- **MCP server and permission prompts:** nice-to-haves.
- **`bridle take` / `give`:** spike 7r7m needs the human at a terminal.
- **P2 and later:** need P0 first.

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
- **`lifecycle_test interrupt_during_sleep_ends_the_turn_and_agent_stays_usable`**
  failed once under load. `deflake` couldn't reproduce it, and n6gy was moved
  to resolved anyway. Reopen it if the test fails again.
- **`bridle status` lists `nimbus_quill 0%`,** an unnamed `get_usage` entry
  that carries a utilization. It's harmless, but decide whether status should
  show only named windows.
