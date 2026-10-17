+++
id = "br-cah9"
title = "Visiting orchestrator token is submit-only: can't ready tickets or comment on same-machine daemons"
kind = "bug"
state = "pending"
created_at = "2026-10-17T02:01:08.052Z"
updated_at = "2026-10-17T02:42:15.404368Z"
created_by = "external:orchestrator@vesta"
watchers = ["external:orchestrator@vesta"]
ticket = "zzt7"
+++

docs/tickets/open/visiting-orchestrator-token-is-submit-only-can-t-ready-ticke-cah9.md

## Thread

### note · external:aide · 2026-10-17T02:42:15.404Z
The human, 2026-10-16 (via aide): "It sounds right yes. Go ahead and" (on the fix below; ready it, high priority).

Cause (aide, from the code): token_pair.rs ~868 mints the same-machine orchestrator a visitor token with --replace-direct because "the holder's old direct token would keep a local orchestrator here, and with one the daemon forwards nothing (br-gr3v)". Only a visitor token records a home, so br-gr3v reused it to get forwarding, and the orchestrator inherited the visitor limits (submit + comment on own) on its own machine's daemons. Before the pair it held direct tokens there with full rights (ready, plan, queue, comment, daemon restart).

Fix the human agreed: keep DIRECT tokens on same-machine daemons (full orchestrator rights, as before) and let a direct token record a home, so the daemon forwards its messages and wakes to <host>-home. No --replace-direct on the same machine; re-pair restores direct tokens where it replaced them. Visitors from other machines keep the visitor limits. Also the two side bugs above (ticket new --project writes into the caller's clone; a refused mint leaves a file).
