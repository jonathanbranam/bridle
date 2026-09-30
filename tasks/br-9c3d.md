+++
id = "br-9c3d"
title = "bridle restart re-resolves the endpoint while waiting (6d5y)"
kind = "bug"
state = "planned"
created_at = "2026-09-30T21:00:35.644Z"
updated_at = "2026-09-30T21:24:00.816010Z"
summary = "restart's wait loop is now wait_for_restart in crates/bridle/src/commands.rs: once the daemon is down it re-resolves the endpoint through discovery on each poll (skipped when --url is explicit), so a changed port no longer times out. The timeout error names <workspace>/.bridle/daemon.log only if the file exists. Two unit tests (fake server on a new port; log named only when given). cli.md doesn't describe the wait, so no change. Ticket 6d5y left open for the manager to resolve."
+++

Fix docs/tickets/open/restart-waits-on-the-old-port-6d5y.md. restart in crates/bridle/src/commands.rs polls health() on the URL it started with; when the restart changes the listen port (a [projects] port or changed [daemon] listen) the daemon comes back elsewhere and the loop reports failure. After the daemon has gone down, re-resolve the endpoint through the normal discovery (registry, [projects] config) on each poll instead of reusing the first URL (keep --url explicit: an explicit URL is used as given). Also make the timeout error not name <workspace>/.bridle/daemon.log unless that file exists. Tests: a fake/stub server test if the existing restart tests allow (look for them), else unit-test the re-resolve step in isolation. CLI only, no daemon code. Docs: cli.md restart if it describes the wait, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: daemon start-up, making restart faster.

## Thread

### note · agent:restart-port · 2026-09-30T21:24:00.816Z
done, but just check is not green: 924/925 pass. The only failure is bridle-spec::spec parses_fast, a timing test that failed 3 runs in a row with host load at 80-120; it passes alone (0.18s) and I didn't touch bridle-spec. Fix: restart's wait (wait_for_restart in commands.rs) re-resolves the endpoint after the daemon goes down, unless --url is given; the timeout error names daemon.log only if the file exists. Added 2 unit tests and a CHANGELOG line. Ticket 6d5y is still open for you to resolve. Commit fca58d3, main merged in.
