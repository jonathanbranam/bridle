---
id: br-t8wk
title: "Aide and advisor sessions get no waiter-grace check: an aide sat with no waiter for ~50h unnoticed"
kind: bug
created: 2026-11-14
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Aide and advisor sessions get the same waiter-grace check the orchestrator has: after the grace period with no waiter (`bridle agent wake`), the daemon records an incident and sends a note to the orchestrator. Today the check (crates/bridle-daemon/src/orchestrator.rs, `waiter_grace`, `absent_since`; config `[orchestrator] waiter_grace`, default 15m) covers only the orchestrator.

What happened (orchestrator@dalek for the human, relayed by orchestrator@vesta, m-0652 on dalek-home): on vesta a DNS outage (2026-11-12 07:41 to at least 11:12 UTC) made the meta-notes aide's API call fail at 08:34:42 ("API Error: Can't reach the API server (EAI_AGAIN)"). Claude Code ended the turn and the aide's waiter was not running. Nothing restarted it or alerted for about 50 hours, until the human typed "Try again" at 2026-11-14 11:00 UTC. `bridle status` showed "no waiter" the whole time. Evidence: meta-notes docs/context/incidents.md, entry 2026-11-12 08:34 (commit cd12073).

## Scope

- Reuse the orchestrator's check for external aide and advisor identities; read the waiter tracking in crates/bridle-daemon/src/wake.rs first. Incident once per absence, cleared when a waiter returns. Note goes to the orchestrator.
- An aide or advisor that was never started, or has gone away on purpose (`bridle agent wake --stop`, session closed), must not raise it: say how the check knows a session is live.
- Not in this ticket (yagni, not yet): typing a nudge into the tagged tmux pane (`@bridle=aide`). Revisit if the incident proves too slow to act on.
- No project files change, so no migration. Docs: where waiter_grace is described in docs/design/agent-host/.

## Acceptance

Tests: aide with no waiter past grace -> one incident and one note; waiter returns -> cleared; stopped waiter or no session -> nothing. `just check` (run with BRIDLE_* env unset). Model: Sonnet.

Same files as the orchestrator-supervisor code in orchestrator.rs; do not run beside another ticket touching it.
