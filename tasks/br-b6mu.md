+++
id = "br-b6mu"
title = "Self-upgrade drain may never restart when it starts during a spawn"
kind = "bug"
state = "planned"
created_at = "2026-10-09T16:54:27.508Z"
updated_at = "2026-10-09T17:20:57.975987Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
ticket = "b6mu"
+++

Ticket: docs/tickets/open/self-upgrade-drain-may-never-restart-when-it-starts-during-a-b6mu.md (read it first; also br-h7gt's thread and docs/context/incidents.md, 2026-10-09 15:56). Model: Sonnet.

Goal: a self-upgrade drain that begins while a spawn is in flight always reaches a quiet point, so the restart always comes.

Where: crates/bridle-daemon/src/server.rs perform_restart (~line 3971; the quiet check at ~3996-4026 waits for no busy agents and `!state.manager.spawning()`), crates/bridle-daemon/src/supervisor.rs (`spawning` counter ~191, `spawning()` ~823, the InFlight guard ~864, the drain refusal ~889, the held first prompt). Test: crates/bridle-daemon/tests/upgrade_test.rs (self_upgrade_restarts_only_after_the_mid_turn_agent_finishes, made deterministic by h7gt with a go-file gate; support in tests/support/mod.rs).

Do:
1. Reproduce first: a test that starts the drain DURING the spawn, with no go-file gate. Show it fails (or document why it cannot) before changing code.
2. Fix in the daemon: either let the agent's held first prompt through during a drain so the spawn completes, or stop counting a spawn as busy once the drain holds it. Pick the smaller, say which in the task comment and in docs/design/agent-host/ (the upgrade/drain section) if behaviour changes.
3. Also handle the side note in the ticket: lifting a drain after a failed restart (perform_restart error path) must deliver the messages held during the drain.
4. Keep the new race test as permanent coverage.

Acceptance: just check passes; the new race test loops clean (30 runs under CPU load, as h7gt did).
Out of scope: other flakes, the load-hold work (n4w4), anything outside drain/spawn/restart code.
Migration: none (no project files or config change).

## Thread

### note · agent:b6mu · 2026-10-09T17:20:19.672Z
CI run 37962490045 (ubuntu, main 233e0c96): 1385 passed, 1 failed: store::tests::cancelled_blocking_task_returns_shutting_down_not_a_panic, panicked at crates/bridle-daemon/src/store.rs:3966:32 ('aborted task should error: 1'). Not upgrade_test and not the drain/spawn race: a different flake (store task cancellation). Out of scope here; worth its own ticket.

### note · external:advisor/product-manager · 2026-10-09T17:20:57.975Z
watching the task
