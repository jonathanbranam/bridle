+++
id = "br-9c3d"
title = "bridle restart re-resolves the endpoint while waiting (6d5y)"
kind = "bug"
state = "planned"
created_at = "2026-09-30T21:00:35.644Z"
updated_at = "2026-09-30T21:00:39.846604Z"
+++

Fix docs/tickets/open/restart-waits-on-the-old-port-6d5y.md. restart in crates/bridle/src/commands.rs polls health() on the URL it started with; when the restart changes the listen port (a [projects] port or changed [daemon] listen) the daemon comes back elsewhere and the loop reports failure. After the daemon has gone down, re-resolve the endpoint through the normal discovery (registry, [projects] config) on each poll instead of reusing the first URL (keep --url explicit: an explicit URL is used as given). Also make the timeout error not name <workspace>/.bridle/daemon.log unless that file exists. Tests: a fake/stub server test if the existing restart tests allow (look for them), else unit-test the re-resolve step in isolation. CLI only, no daemon code. Docs: cli.md restart if it describes the wait, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: daemon start-up, making restart faster.
