+++
id = "br-bek3"
title = "The gateway runs detached and keeps itself current, like the daemon, on dalek and on client machines"
kind = "feature"
state = "planned"
created_at = "2026-10-04T21:27:39.949Z"
updated_at = "2026-10-04T21:42:33.030508Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
priority = "high"
+++

original id: bek3
Build docs/tickets/open/the-gateway-runs-detached-and-keeps-itself-current-like-the-bek3.md (read it, plus docs/design/agent-host/daemon.md 'Running it' and 'Automatic upgrade', docs/design/human-web-ui.md build task 10, tickets jmpf and chvf). The human: 'spec'd out and running': SPEC FIRST. Step 1 (commit first): a short design/spec section in docs/design/human-web-ui.md (or a new gateway doc if cleaner) saying how the gateway is run detached (bridle gateway --detach, same pattern as bridle serve --detach: re-exec in a new process group, log to ~/.bridle/gateway.log, wait for health, refuse a second), how it stays current (restart onto the new bridle binary when the daemon upgrades, like the daemon's in-place restart), and the per-machine [gateway] config (bind address/port, whether it runs; extends jmpf's section). Same solution on dalek and on client machines. Step 2: implement it to that spec in crates/bridle-gateway and crates/bridle (gateway subcommand, daemon upgrade hook), reusing the daemon's detach code rather than copying it; update cli.md. Config defaults only; do not edit ~/.bridle/config.toml. Acceptance: just check passes; tests for detach (starts, logs, health, second start refused) and restart-on-upgrade with a fake. Migration: a new optional [gateway] key must have a default so existing configs keep working; say so in the doc. Model: Sonnet. Size M; if too big, land the spec plus detach first and report restart-on-upgrade for tc7t. Out of scope: the UI install (bridle-ui project, see br-tc7t) and multi-machine gateway (gateway 9/10, br-7172).

## Thread

### note · external:orchestrator · 2026-10-04T21:27:43.723Z
Approval: the human, 2026-10-04 via aide (m-0141): "let's get the work spec'd out and running also for running the gateway detached in the background the same way the daemon runs." Full quote in the ticket (ece4265e).

### note · agent:pm-1 · 2026-10-04T21:28:07.474Z
priority: normal -> high

### note · external:aide · 2026-10-04T21:42:33.030Z
watching the task
