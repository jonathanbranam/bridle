# Orchestrator state

The orchestrator's working notes for handing over between sessions. The role
itself is in `.bridle/roles/orchestrator.md`. Update this file whenever the
queue, open items or decisions change. Last updated 2026-09-27 21:45 UTC, at
the end of the first self-hosted session.

## In flight

The manager is `manager-1` (Sonnet). Its context is long after ~50 turns; if
it degrades, ask the human to stop it and spawn a fresh `bridle spawn manager`.

Its queue, two workers at a time:

1. **`mcp-deferral`** (docs, running): mark the MCP server and permission
   prompts deferred in `docs/proposal/build-order.md`; add the human's
   tentative connector preference to ticket u6wk.
2. **(A) Usage reporting**
   ([usage and budget](docs/design/usage-and-budget.md)):
   - the ledger's role and model columns;
   - `bridle usage --by role|model|agent --since`;
   - the cache-hit ratio per role.
3. **(B) `bridle cost audit [--check]`**, with a committed
   `.bridle/cost-baseline.json`. Runs in parallel with (A).
4. **Resolve c7eb and c5a8** (docs). The human agreed on 2026-09-27: task
   records on a state branch; questions in the blocking task's thread, with a
   SQLite index for the inbox.
5. **P0** ([build order](docs/proposal/build-order.md)). The manager is to send
   its task breakdown before spawning. It takes priority over (C).
6. **(C) The governor's `[models]` step-down** per role, after (A), because
   they touch the same files.

Parked: `bridle/mcp-1` (WIP `cae932c`, a hand-rolled read-side `/mcp`, not
tested). Don't merge it.

## Waiting on the human

- **Optional:** set `bridle statusline` as the `statusLine` command in
  `~/.claude/settings.json`. Its parser is unverified against the real JSON
  (spike m9wt).

## Deferred

- **MCP server and permission prompts:** nice-to-haves (see above).
- **`bridle take` / `give`:** spike 7r7m needs the human at a terminal.
- **P2 and later:** need P0 first.

## Done on 2026-09-27 (all merged into `main`, all verified green)

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
