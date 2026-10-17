+++
id = "br-ux3h"
title = "Waiter: the upgrade exit (code 6) says when (UTC), from/to version and expected downtime"
kind = "feature"
state = "open"
created_at = "2026-10-17T01:14:02.608Z"
updated_at = "2026-10-17T01:15:27.280183Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "low"
priority_at = "2026-10-17T01:14:52.014911Z"
ticket = "ux3h"
+++

docs/tickets/open/waiter-the-upgrade-exit-code-6-says-when-utc-from-to-version-ux3h.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T01:14:52.014Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-17T01:14:52.409Z
The human, 2026-10-16 ~7:30 PM ET, to advisor (product-manager): "Great yeah seems like a small useful enhancement. But low priority can be done tonight". Readied, priority low; fine to build tonight.

### note · agent:pm-1 · 2026-10-17T01:15:27.280Z
Brief (pm-1): the ask is the brief. Model: Sonnet. Files: crates/bridle-daemon/src/server.rs (set_stop_reason and wherever the upgrade stop reason is built), the wake CLI that prints the exit-6 line and the wire type for the stop reason (crates/bridle-api/src/types.rs, change daemon and clients together), docs/design/cli.md, docs/design/agent-host/daemon.md, CHANGELOG. "expected_back_secs" comes from the last recorded restart duration if one exists; omit it when unknown. Acceptance: just check; tests for text and --json forms and exit code 6. Low priority; it follows the queue. Same wake code as br-8kzq: blocked-by br-8kzq.
