+++
id = "br-sftn"
title = "CI red on main after br-b2g8: check (ubuntu-latest) fails on 9e2b4858"
kind = "fix"
state = "integrated"
created_at = "2026-10-20T23:15:41.625Z"
updated_at = "2026-10-20T23:59:33.264845Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:ci-sftn",
]
branch = "bridle/ci-sftn"
commit = "9bd67d61b5b36dd0c7d64d7d145881f815b64805"
summary = 'Disabled the flaky governor test working_agent_is_notified_then_stopped_when_its_turn_ends with #[ignore = "flaky, br-remg"] (crates/bridle-daemon/tests/governor_test.rs), plus a CHANGELOG Internal line. Nothing else changed. Landed on diff review and CI, not the local check: just check here fails on load-dependent daemon-timeout tests that pass alone (serve_detach_returns_once_healthy_and_daemons_lists_it, two_sites_log_in_serve..., ticket_row_commands_match_the_hidden_task_alias, sigint_shuts_down_cleanly_while_idle). Those are filed as br-6wp2; the governor fix is br-remg.'
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

### note · agent:ci-sftn · 2026-10-20T23:58:23.025Z
Retry of just check: exit 100 (recipe test failed), Summary 1186/1941 tests run: 1185 passed, 1 failed, 7 skipped. Fails only on load timeouts that pass alone: (1) cli_e2e serve_detach_returns_once_healthy_and_daemons_lists_it: 'detach json: EOF while parsing' (passes alone, 29s); earlier runs had (2) dev_site_test two_sites_log_in_serve_the_fixtures_and_go_down_one_at_a_time (acme-docs daemon 30s wait, passes alone 14s), (3) cli_e2e ticket_row_commands_match_the_hidden_task_alias (daemon.json wait, passes alone), (4) cli_e2e sigint_shuts_down_cleanly_while_idle (latest prior run). Not green, so no push or built mark. Commit 0bcec9fb unchanged. Taking this to the orchestrator per your m-17805 is your call.

### note · agent:manager-2 · 2026-10-20T23:58:26.869Z
manager-2: worker commit 0bcec9fb is the trivial disable (governor test #[ignore] + CHANGELOG). just check on this machine fails every run on a different load-dependent daemon-timeout test that passes alone (serve_detach_returns_once_healthy_and_daemons_lists_it, two_sites_log_in_serve..., ticket_row_commands_match_the_hidden_task_alias, sigint_shuts_down_cleanly_while_idle); exit 100, 1185/1186 passed. Landing runs the check, so it would fail the same way. Asking the orchestrator: accept landing on a diff review plus CI as the check, or hold until the machine is quiet? Not landing meanwhile.

### note · external:orchestrator · 2026-10-20T23:59:15.348Z
orchestrator: yes, land it. I reviewed 0bcec9fb: one #[ignore = "flaky, br-remg"] line plus a CHANGELOG line, nothing else. main is already red, so skip the local check for this landing and let CI on the push verify it. Don't ignore the other four tests; their load-dependent timeouts are filed as br-6wp2 for later.

### note · agent:manager-2 · 2026-10-20T23:59:21.511Z
Orchestrator approved landing on diff review with CI as the check. Please write the ticket summary now (bridle ticket summary), then push-branch if not done and message me. Do not rerun just check.

### note · agent:ci-sftn · 2026-10-20T23:59:26.019Z
built: 0bcec9fb (branch bridle/ci-sftn)

### note · agent:ci-sftn · 2026-10-20T23:59:29.069Z
done: disabled governor_test working_agent_is_notified_then_stopped_when_its_turn_ends (#[ignore = "flaky, br-remg"]) + CHANGELOG Internal line; local just check not run per orchestrator (fails on load timeouts that pass alone, filed as br-6wp2); CI on the push verifies; 0bcec9fb

### note · agent:manager-2 · 2026-10-20T23:59:33.264Z
integrated: 9bd67d61b5b36dd0c7d64d7d145881f815b64805 (branch bridle/ci-sftn)
