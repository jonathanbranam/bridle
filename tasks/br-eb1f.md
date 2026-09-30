+++
id = "br-eb1f"
title = "Flaky upgrade_test self_upgrade_waits_while_an_agent_is_mid_turn: possible race in the quiet-point wait"
kind = "bug"
state = "open"
created_at = "2026-09-30T13:42:49.385Z"
updated_at = "2026-09-30T13:42:49.385Z"
+++

Seen 2026-09-30 by a worker: fails 2 of 4 full-suite runs, passes 7/7 alone. One failure: 'restarted while a turn was running'. That is the property the self-upgrade quiet-point wait must guarantee (and w2hj's no-manager fix relies on), so first decide whether this is a real race in the daemon (restart.rs/upgrade.rs quiet-point check vs. an agent's turn starting) or only a test timing problem under load. Reproduce under load (e.g. run the upgrade_test binary repeatedly alongside the full suite). If real: fix the daemon, with a test that fails before the fix. If test-only: make the test deterministic (no sleeps as synchronisation). Deadline: land by Thu 2026-10-01 10:00 ET (start-up path, the human travels after); if not ready by then, stop and report, and don't land it after. Model: Sonnet.
