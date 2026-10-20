+++
id = "br-6wp2"
title = "just check fails on dalek under load: four daemon startup/shutdown-wait tests time out (each passes alone)"
kind = "fix"
state = "open"
created_at = "2026-10-20T23:59:08.335Z"
updated_at = "2026-10-20T23:59:34.432786Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "6wp2"
+++

docs/tickets/open/just-check-fails-on-dalek-under-load-four-daemon-startup-shu-6wp2.md

submitted by external:orchestrator

## The ask

Filed by the orchestrator, 2026-10-20 ~8:00 PM ET. Fix later, after features ship (the human, 2026-10-11, flaky-test rule). Disable any of these that turns CI on main red.

On dalek, `just check` failed on every run during br-sftn (ci-sftn, 2026-10-20 23:36-23:58Z), each time on a different daemon startup or shutdown wait. Each test passed when run alone:

- cli_e2e serve_detach_returns_once_healthy_and_daemons_lists_it ("detach json: EOF while parsing")
- dev_site_test two_sites_log_in_serve_the_fixtures_and_go_down_one_at_a_time (acme-docs daemon 30 s wait)
- cli_e2e ticket_row_commands_match_the_hidden_task_alias (waits for daemon.json)
- cli_e2e sigint_shuts_down_cleanly_while_idle

Load was high (up to ~45 on 16 cores, worker builds). A local check that can't pass blocks every landing. Find why these waits are load-sensitive (fixed timeouts on a busy machine, polling, contention on shared ports or dirs) and make them reliable. Check under load as br-n96z did.

Model: Sonnet. Related: br-remg, br-sftn, br-n6gy.

## Thread

### note · external:orchestrator · 2026-10-20T23:59:08.335Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-20T23:59:34.432Z
orchestrator: readied (fix).
