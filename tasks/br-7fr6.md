+++
id = "br-7fr6"
title = "Cap test threads in just test / just check so a full run does not drive load past the governor"
kind = "bug"
state = "planned"
created_at = "2026-10-08T18:16:43.505Z"
updated_at = "2026-10-08T23:22:55.498618Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Asked by manager-2 (m-7188), approved by the orchestrator (m-7187). It blocks every land under load.

Problem: a full `just check` (cargo nextest run --workspace, default test-threads = number of cores, 16 here) drives host load to 45+ on 16 cores, and the load makes timing-sensitive tests fail at random: governor_test working_agent_is_notified..., upgrade_test a_long_drain_wakes_the_orchestrator_once, context_governor_test many_concurrent... Many tests start their own daemon, so the threads multiply (see also br-3p3h and br-9z2n, which cut the ps forks; this task is independent of them).

Goal: a full `just check` keeps the 1-minute load average under the governor threshold (2.5/core, 40 on 16 cores) on this machine, with no flaky timeouts, without making the run much slower.

Change:
- Set a thread cap for nextest. Put it in .config/nextest.toml ([profile.default] test-threads = N) so every nextest caller gets it (just test, just check, check-affected, the workers), not only one recipe. Choose N by measurement: try 4, 6 and 8 on a 16-core machine, and pick the largest N that keeps load under 40 with wall time within about 1.5x of the uncapped run. A "num-cpus"-relative value (e.g. test-threads = "num-cpus/2") is acceptable if nextest supports it in the pinned version; otherwise a fixed number.
- Do not touch individual tests or their timeouts.

Files: .config/nextest.toml, justfile only if needed, CHANGELOG.md (entry on top), and a line in CLAUDE.md Commands only if the behaviour a worker sees changes (it should not).

Acceptance: just check passes twice in a row. In the done note give a table: N, peak 1-minute load (uptime sampled during the run), wall time, failures, for uncapped and each N tried. If the machine is busy with other agents, say so and note the load before starting.

Model: Sonnet. Migration: none (repo config only). Out of scope: the governor threshold, per-test timeouts, ps forks (br-9z2n), CI.

## Thread

### note · agent:manager-2 · 2026-10-08T22:55:43.176Z
manager-2: FYI another worker (daemondocs, br-5e4k) started; it will build but runs the full check once, at its end. Note the extra load in your measurements and rerun a measurement if it is disturbed.

### note · agent:threadcap · 2026-10-08T23:22:55.498Z
threadcap: machine never went quiet (load 31-100 before each run, 148 now; other agents building/testing). All 1336 tests passed every run. N=4 peak 154 wall 446s; N=6 peak 110 wall 259s; N=8 peak 102 wall 173s; uncapped(16) peak 175 wall 144s. Peaks are dominated by other agents so do not separate the Ns; wall times do (4: 3.1x, 6: 1.8x, 8: 1.2x of uncapped). Proposal N=8. Asked manager-2 whether to accept or re-measure when quiet.
