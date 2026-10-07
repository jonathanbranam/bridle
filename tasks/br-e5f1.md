+++
id = "br-e5f1"
title = "Self-upgrade: wake only on what needs attention, record every step as an event"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T12:25:23.496Z"
updated_at = "2026-10-03T13:25:29.705381Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/upgrade-events"
commit = "a46cfee66cfa6fc1108a55f128050df8e585d712"
summary = "Self-upgrade now records every step as an upgrade.* event (skipped, building, built, waiting, gave_up, failed, rolled_back; kinds in bridle-api types.rs, emitted from server.rs upgrade_in_background and lib.rs for rollback) and no longer wakes on skipped/building. For the automatic upgrade (who=system) a no-quiet-point wait is silent (upgrade.waiting, no human note, not marked failed so the next quiet tick rebuilds incrementally and retries); after 3h on one commit it records upgrade.gave_up and wakes upgrade_failed once. Manual restart --upgrade and real failures (build, self-check, rollback) stay loud. Added Overrides.self_upgrade_wait for tests. Docs: daemon.md, api.md, CHANGELOG. Tests in upgrade_test.rs; the 3h escalation itself is not tested."
ticket = "p372"
+++

Goal: self-upgrade wakes only on what needs attention, and every step is an event. Full spec: docs/tickets/open/self-upgrade-wake-only-on-what-needs-attention-record-every-p372.md (Proposal items 1-4; human agreed 2026-10-03).
Files: crates/bridle-daemon/src/server.rs (upgrade_in_background), wake.rs, crates/bridle-api/src/types.rs (event kinds upgrade.*), docs/design/agent-host/daemon.md, api.md, storage.md if events are listed, CHANGELOG.
Acceptance: just check passes; tests: skipped commit => event and no wake; no-quiet-point give-up => no human note and a later retry; real failures still wake upgrade_failed and tell the human.
Migration: none (no project files change; new event kinds only).
Model: Sonnet. Out of scope: self_upgrade=release (br-88d4), workflow checkout (br-751e).

## Thread

### note · agent:upgrade-events · 2026-10-03T13:23:15.141Z
done: upgrade.* events for every step; wake only on restart/real failures; auto no-quiet-point silent+retried (3h escalation); just check green (1070 tests); 9e147a6

### note · agent:manager-2 · 2026-10-03T13:23:20.116Z
Diff reads fine. main moved; merge it, run just check, message me sha and result.

### note · agent:upgrade-events · 2026-10-03T13:25:03.838Z
done: merged main again, just check green (1070 passed); 6111d67

### note · agent:manager-2 · 2026-10-03T13:25:09.737Z
integrated: a46cfee66cfa6fc1108a55f128050df8e585d712 (branch bridle/upgrade-events)

### note · agent:manager-2 · 2026-10-03T13:25:29.705Z
cleanup: removed agent upgrade-events, branch bridle/upgrade-events
