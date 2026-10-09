+++
id = "br-b6mu"
title = "Self-upgrade drain may never restart when it starts during a spawn"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T16:54:27.508Z"
updated_at = "2026-10-09T18:14:46.707007Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
branch = "bridle/b6mu"
commit = "3323d2f4acc8d9a75787616c430004635515c4e6"
summary = "Reproduced (upgrade_test a_drain_starting_during_a_spawn_restarts_promptly, no go-file gate; failed at 10.3 s): a drain that begins during a spawn holds the first prompt, no init comes, and the spawn sat out its 8 s SPAWN_READY_TIMEOUT before the quiet point. So the cause is a delay, not a permanent hang; it cannot explain a 60 s non-restart by itself. Fix (the smaller option): spawn no longer waits for readiness when its first message was held (supervisor.rs). Also new AgentManager::lift_drain, used by perform_restart's error path, delivers each running agent's oldest held message. Docs: daemon.md Restart in place, CHANGELOG. The lift_drain error path has no dedicated test (no cheap way to make drain_and_restart fail). Looped the new test 30 of 30 on macOS. Unrelated CI flake seen on 233e0c96: store::tests::cancelled_blocking_task_returns_shutting_down_not_a_panic."
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

### note · agent:b6mu · 2026-10-09T17:48:29.240Z
done: drain-during-spawn race reproduced and fixed (spawn skips the readiness wait when its prompt is held; lift_drain delivers held messages); just check exit 0; 72c42a8d full 72c42a8dad48e03e93c98dea6387067939e5a2a6; new test looped 30/30

### note · external:orchestrator · 2026-10-09T17:49:25.498Z
From orchestrator: read the diff (72c42a8d): approved to land. Small and correct; the held-prompt check and lift_drain match daemon.md. Noted that this explains an 8 s delay, not CI's 60 s non-restart: the ticket's resolution should say that cause stays unknown, to reopen only if a restart-never-comes symptom recurs.

### note · agent:manager-2 · 2026-10-09T18:14:08.208Z
integrated: 3323d2f4acc8d9a75787616c430004635515c4e6 (branch bridle/b6mu)

### note · agent:manager-2 · 2026-10-09T18:14:46.707Z
cleanup: removed agent b6mu, branch bridle/b6mu
