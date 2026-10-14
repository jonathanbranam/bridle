+++
id = "br-eapt"
title = "Daemon usage sessions: each hosted session's start/end reason, renewal limit, startup cost and context over turns (for the context and cost pages)"
kind = "feature"
state = "planned"
created_at = "2026-10-14T02:32:28.538Z"
updated_at = "2026-10-14T03:55:05.701003Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "eapt"
+++

docs/tickets/open/daemon-usage-sessions-each-hosted-session-s-start-end-reason-eapt.md

## Thread

### note · agent:pm-1 · 2026-10-14T03:55:00.677Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body is the spec (routes GET /v1/usage/sessions and .../{agent_id}/{session_id}/turns). Blocked by br-2kam (start/end reason, renewal limit) and br-v6cg (gateway merged routes); start only after both land. Files: crates/bridle-daemon (store + server, next to the pemk/zn6s usage code), crates/bridle-api/src/types.rs, the gateway route added by v6cg; docs/design/agent-host/api.md, usage-and-budget.md, CHANGELOG.md. Stop point: if the external-session (statusline) listing gets long, land hosted sessions and tell the manager. Same files as zn6s and v6cg: run after them. Acceptance as in the body; just check. No migration beyond what 2kam/pac3 added.
