---
id: br-6wp2
title: "just check fails on dalek under load: four daemon startup/shutdown-wait tests time out (each passes alone)"
kind: fix
created: 2026-10-20
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

On dalek, `just check` failed on every run during br-sftn (2026-10-20 23:36-23:58Z), each time on a different daemon startup or shutdown wait. Each test passed when run alone:

- cli_e2e serve_detach_returns_once_healthy_and_daemons_lists_it ("detach json: EOF while parsing")
- dev_site_test two_sites_log_in_serve_the_fixtures_and_go_down_one_at_a_time (acme-docs daemon 30 s wait)
- cli_e2e ticket_row_commands_match_the_hidden_task_alias (waits for daemon.json)
- cli_e2e sigint_shuts_down_cleanly_while_idle

Load was high (up to ~45 on 16 cores, worker builds). A local check that can't pass blocks every landing. Find why these waits are load-sensitive (fixed timeouts on a busy machine, polling, contention on shared ports or dirs) and make them reliable. If one turns CI on main red, #[ignore] it naming this ticket (as br-sftn did).

Check: `just check` passes under load as br-n96z checked (run it alone and while a build runs). Model: Sonnet. Files: crates/bridle/tests/cli_e2e.rs, dev_site_test, shared test support. Related: br-remg, br-sftn, br-n6gy. Same test-support files as br-remg: run one after the other.
