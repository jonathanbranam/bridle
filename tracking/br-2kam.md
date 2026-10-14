+++
id = "br-2kam"
title = "Each hosted session records how it started and ended, and the renewal limit in force (ktgd decision 7)"
kind = "feature"
state = "claimed"
created_at = "2026-10-13T23:05:26.890Z"
updated_at = "2026-10-14T03:56:04.598173Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:2kam",
]
ticket = "2kam"
+++

docs/tickets/open/each-hosted-session-records-how-it-started-and-ended-and-the-2kam.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:30.223Z
Brief (pm-1). Model: Sonnet. Size: S. Blocked by br-pac3 (turns carry the session id). The ask and ktgd decision 7 are the spec. Prefer a small `agent_sessions` table (agent id, session id, started_at/start_reason, ended_at/end_reason, end_tokens, renewal_limit) over events, if events would need parsing to query; one DB migration (storage.md), written from the supervisor's spawn/restart/renew/resume/handover/stop paths (find them in crates/bridle-daemon/src/supervisor.rs; the renewal limit is the configured context threshold). Rows are written by existing code paths: no new polling. Test: a renewal shows one session ending (renewal, at N tokens, limit L) and the next starting (renewal). Docs: storage.md, usage-and-budget.md, CHANGELOG.md. Out of scope: any report or query on top of it.
