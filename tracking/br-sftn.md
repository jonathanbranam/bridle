+++
id = "br-sftn"
title = "CI red on main after br-b2g8: check (ubuntu-latest) fails on 9e2b4858"
kind = "fix"
state = "claimed"
created_at = "2026-10-20T23:15:41.625Z"
updated_at = "2026-10-20T23:56:23.958795Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:ci-sftn",
]
ticket = "sftn"
+++

Critical: main is red (the human, 2026-10-03: anything that breaks CI on main is critical).

CI run https://github.com/jonathanbranam/bridle/actions/runs/39512201248 on 9e2b4858, check (ubuntu-latest). The only failure is the known flaky governor test, not br-b2g8's change:

    governor_test working_agent_is_notified_then_stopped_when_its_turn_ends
    panicked at crates/bridle-daemon/tests/support/mod.rs:393:13
    timed out waiting for agent a-2xysa to reach Stopped

Its fourth flake (jtn8, n96z, ar4d). Per the human's rule (2026-10-11: disable a flaky test and file a ticket for later), disable it: add `#[ignore = "flaky, br-remg"]` to that test in `crates/bridle-daemon/tests/governor_test.rs`, the way br-ck8a did. Nothing else. The real fix is br-remg, scheduled later.

Acceptance: `just check` green; then CI green on main. Model: Haiku.

## Thread

### note · external:orchestrator · 2026-10-20T23:16:50.537Z
orchestrator: rewritten as the #[ignore] disable (known flaky test, 4th time; fix is br-remg, later). Readied; critical, red main.

### note · agent:ci-sftn · 2026-10-20T23:36:45.267Z
ci-sftn handoff: governor_test working_agent_is_notified_then_stopped_when_its_turn_ends is #[ignore = "flaky, br-remg"]; CHANGELOG Internal line added (uncommitted). just check is running in the background (log /tmp/br-sftn-check.log). Next: confirm exit 0 and test count, commit, push-branch, ticket built, message manager-2. If the check fails, I report it and keep the claim until the manager decides.

### note · agent:ci-sftn · 2026-10-20T23:56:07.863Z
br-sftn is NOT done: the disable commit 0bcec9fb is in place (governor test #[ignore] + CHANGELOG Internal line, nothing else), but just check is still red on this machine. Three runs, three different daemon-timeout tests, each passing when run alone: dev_site_test two_sites_log_in_serve... (30s acme-docs daemon wait), cli_e2e ticket_row_commands_match_the_hidden_task_alias (waits for daemon.json), cli_e2e sigint_shuts_down_cleanly_while_idle (latest run). All are load-dependent startup/shutdown waits, not the governor test. Options: (a) you accept that the load flake keeps failing ubuntu-equivalent runs and I file it on br-remg; (b) I ignore these too, touching more tests than the ticket allows, which needs your OK; (c) you run the check on a quieter machine or in CI. I have NOT pushed, marked built, or written a done summary. Tell me which.

### note · agent:ci-sftn · 2026-10-20T23:56:23.958Z
ci-sftn status: disable commit 0bcec9fb in place. Retry of just check running (log /tmp/br-sftn-check.log); not yet green, so no done report, push or built mark. Manager asked (m-17805) for the three load-dependent test names and exit status if it still fails only on timeouts that pass alone.
