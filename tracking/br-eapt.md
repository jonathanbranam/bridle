+++
id = "br-eapt"
title = "Daemon usage sessions: each hosted session's start/end reason, renewal limit, startup cost and context over turns (for the context and cost pages)"
kind = "feature"
state = "integrated"
created_at = "2026-10-14T02:32:28.538Z"
updated_at = "2026-10-15T04:01:35.859221Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:eapt",
]
branch = "bridle/eapt"
commit = "6f7123680f7c1a1138bb61ed86e202d4922c0083"
summary = "Added GET /v1/usage/sessions?since= (UsageSessions) and GET /v1/usage/sessions/{agent_id}/{session_id}/turns: store usage_session_list joins turns (by session_id, NULL-session turns left out) to agent_sessions for reasons and renewal limit; startup = first turn; peak and end context; external statusline sessions listed with null turns/reasons/startup. A session with any turn in the period is summed whole; a resumed session id is one row (start fields from first agent_sessions row, end from last). Gateway: new session_usage.rs serves /api/v1/usage/sessions across projects (unreachable listed, never zero) and /api/v1/projects/{p}/usage/sessions/{agent}/{session}/turns; bindings regenerated. Test: agent renewed once gives two sessions with right reasons, limit, startup, peak. Docs: api.md, usage-and-budget.md, human-web-ui.md, CHANGELOG."
ticket = "eapt"
+++

docs/tickets/open/daemon-usage-sessions-each-hosted-session-s-start-end-reason-eapt.md

## Thread

### note · agent:pm-1 · 2026-10-14T03:55:00.677Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body is the spec (routes GET /v1/usage/sessions and .../{agent_id}/{session_id}/turns). Blocked by br-2kam (start/end reason, renewal limit) and br-v6cg (gateway merged routes); start only after both land. Files: crates/bridle-daemon (store + server, next to the pemk/zn6s usage code), crates/bridle-api/src/types.rs, the gateway route added by v6cg; docs/design/agent-host/api.md, usage-and-budget.md, CHANGELOG.md. Stop point: if the external-session (statusline) listing gets long, land hosted sessions and tell the manager. Same files as zn6s and v6cg: run after them. Acceptance as in the body; just check. No migration beyond what 2kam/pac3 added.

### note · agent:eapt · 2026-10-15T03:50:23.831Z
done: daemon+gateway session usage routes; just check exit 0, 1708 tests passed; aca63ed6 (check ran on d5a5c56d; merge of main added only a ticket doc)

### note · agent:manager-2 · 2026-10-15T03:50:28.803Z
built: aca63ed6 (branch bridle/eapt)

### note · agent:manager-2 · 2026-10-15T04:01:15.473Z
integrated: 6f7123680f7c1a1138bb61ed86e202d4922c0083 (branch bridle/eapt)

### note · agent:manager-2 · 2026-10-15T04:01:35.859Z
cleanup: removed agent eapt, branch bridle/eapt
