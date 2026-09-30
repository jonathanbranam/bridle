+++
id = "br-eb1f"
title = "Flaky upgrade_test self_upgrade_waits_while_an_agent_is_mid_turn: possible race in the quiet-point wait"
kind = "bug"
state = "planned"
created_at = "2026-09-30T13:42:49.385Z"
updated_at = "2026-09-30T14:00:08.003022Z"
summary = "Real daemon race, not just test timing: spawn() sets an agent Idle before its first system/init flips it to Working, so the quiet-point check (self_upgrade_tick and perform_restart in server.rs) could pass while the first turn was already on the way, then restart shortly after the agent showed Working. Added an in-flight-spawn counter on AgentManager (spawning()), read before the agent list in both checks, that counts as busy. Test now polls flag-then-state instead of a fixed 1s sleep. Not shown failing pre-fix (couldn't force the window deterministically); 6 repeated runs of upgrade_test pass. Remaining known gap: a message sent later to an Idle agent has the same idle-until-init window."
+++

Seen 2026-09-30 by a worker: fails 2 of 4 full-suite runs, passes 7/7 alone. One failure: 'restarted while a turn was running'. That is the property the self-upgrade quiet-point wait must guarantee (and w2hj's no-manager fix relies on), so first decide whether this is a real race in the daemon (restart.rs/upgrade.rs quiet-point check vs. an agent's turn starting) or only a test timing problem under load. Reproduce under load (e.g. run the upgrade_test binary repeatedly alongside the full suite). If real: fix the daemon, with a test that fails before the fix. If test-only: make the test deterministic (no sleeps as synchronisation). Deadline: land by Thu 2026-10-01 10:00 ET (start-up path, the human travels after); if not ready by then, stop and report, and don't land it after. Model: Sonnet.

## Thread

### note · agent:upgrade-flake · 2026-09-30T14:00:04.244Z
done: real race (spawn reads idle until first init, so quiet check passed mid-spawn); quiet checks now count in-flight spawns as busy, test made deterministic; just check passes (907 tests); not shown failing pre-fix; 5c01c50

### note · agent:manager-2 · 2026-09-30T14:00:08.003Z
Good diagnosis. main moved: merge main into your branch, rerun just check, and message me right away. Deadline Thu 10:00 ET.
