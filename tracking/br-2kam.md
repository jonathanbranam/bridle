+++
id = "br-2kam"
title = "Each hosted session records how it started and ended, and the renewal limit in force (ktgd decision 7)"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T23:05:26.890Z"
updated_at = "2026-10-14T06:23:50.253569Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:2kam",
]
branch = "bridle/2kam"
commit = "808b6f916e7aa7d7f0bfb8bab458dc4de41677e8"
summary = "New agent_sessions table (SCHEMA_V30): one row per hosted session start (spawn/resume/renewal) with ended_at/end_reason (renewal, stop, shutdown or exit reason), end_tokens (context size at the end of the last turn, so the handoff turn's reading, not the crossing) and renewal_limit (wind_down_at for the role at start). Written from register_and_start, renew_inner and the exit handler; first ender wins so renew's 'renewal' is not overwritten by the process exit. No handover-specific end reason (handover does not end a session; the renewal that follows does). Docs: storage.md, usage-and-budget.md, CHANGELOG. Test in context_governor_test.rs."
ticket = "2kam"
+++

docs/tickets/open/each-hosted-session-records-how-it-started-and-ended-and-the-2kam.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:30.223Z
Brief (pm-1). Model: Sonnet. Size: S. Blocked by br-pac3 (turns carry the session id). The ask and ktgd decision 7 are the spec. Prefer a small `agent_sessions` table (agent id, session id, started_at/start_reason, ended_at/end_reason, end_tokens, renewal_limit) over events, if events would need parsing to query; one DB migration (storage.md), written from the supervisor's spawn/restart/renew/resume/handover/stop paths (find them in crates/bridle-daemon/src/supervisor.rs; the renewal limit is the configured context threshold). Rows are written by existing code paths: no new polling. Test: a renewal shows one session ending (renewal, at N tokens, limit L) and the next starting (renewal). Docs: storage.md, usage-and-budget.md, CHANGELOG.md. Out of scope: any report or query on top of it.

### note · agent:2kam · 2026-10-14T05:26:50.473Z
built: 247f8966e90e39deb5413980af8a396ef16bf7c2 (branch bridle/2kam)

### note · agent:2kam · 2026-10-14T05:26:50.514Z
done: agent_sessions table + writes on start/renew/exit; just check exit 0, 1659 tests passed, on merged tip 247f8966e90e39deb5413980af8a396ef16bf7c2

### note · agent:manager-2 · 2026-10-14T06:22:49.836Z
integrated: 808b6f916e7aa7d7f0bfb8bab458dc4de41677e8 (branch bridle/2kam)

### note · agent:manager-2 · 2026-10-14T06:23:36.076Z
cleanup: removed agent 2kam, branch bridle/2kam

### note · external:advisor/product-manager · 2026-10-14T06:23:50.253Z
Resolved by PdM: verified 808b6f91 (agent_sessions: start spawn/resume/renewal; end renewal/stop/shutdown/exit with end_tokens and renewal_limit; renewal test). Handover is not its own end reason because the renewal after it ends the session; restart shows as resume. Fine for eapt.
