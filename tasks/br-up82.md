+++
id = "br-up82"
title = "Self-upgrade refuses a good build: the new binary's self-check timed out (60 s) on the Intel Mac under load"
kind = "bug"
state = "planned"
created_at = "2026-10-05T04:02:26.820Z"
updated_at = "2026-10-05T06:20:38.630149Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
summary = "check_built (crates/bridle-daemon/src/upgrade.rs) now runs the new binary's self-check up to 3 times, each capped at 60 s, retrying only on timeout; a real non-zero exit fails at once with the stderr tail, and all-timeouts refuses with 'timed out (3 attempts of 60 s)'. Chose retry over warm-up as smaller. syspolicyd cause noted as a comment only. Tests: slow-first-then-pass, always-timeout, failing-not-retried. daemon.md and CHANGELOG updated."
+++

original id: up82
Ticket (the ask and options; read first): docs/tickets/open/self-upgrade-refuses-a-good-build-the-new-binary-s-self-chec-up82.md
Goal: a good build must not fail the upgrade self-check just because it is the binary's first, slow run on a loaded Intel Mac. Fix in crates/bridle-daemon upgrade.rs `check_built` (60 s cap on `bridle serve --check`): retry the check once or twice before failing, or warm the binary first (`bridle --version`, own generous timeout). Pick the smaller; keep failure clear when it really fails. Don't try to confirm the syspolicyd guess beyond a comment if cheap.
Acceptance: just check passes; a unit test where the first check times out and the second passes upgrades, and where all fail it still refuses with the reason. Update the upgrade doc under docs/design/agent-host/ if it describes the check.
Model: Sonnet. Out of scope: signing setup (p88z), the CLI/daemon version skew side effect.

## Thread

### note · agent:upgrade-selfcheck · 2026-10-05T06:20:38.630Z
done: self-check retries on timeout (3 runs), tests + doc + changelog; just check exit 0, 1204 tests passed, on db3a4815. Then merged main (only brings ksz7's cli_e2e.rs change, disjoint), tip ccd916fa not re-checked.
