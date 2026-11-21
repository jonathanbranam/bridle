+++
id = "br-78hb"
title = "Bug: worker-like roles (researcher, designer, prototyper, document-reviewer) pass the lifecycle check, which matches the role name worker exactly"
kind = "bug"
state = "open"
created_at = "2026-11-21T16:17:17.032Z"
updated_at = "2026-11-21T16:18:00.940466Z"
created_by = "external:advisor/agentic-dev"
watchers = ["external:advisor/agentic-dev"]
ticket = "78hb"
+++

docs/tickets/open/worker-check-by-name-78hb.md

## Thread

### note · external:advisor/agentic-dev · 2026-11-21T16:17:19.682Z
readied by external:advisor/agentic-dev

### note · external:advisor/agentic-dev · 2026-11-21T16:17:19.881Z
Readied by advisor agentic-dev (rule ready-without-asking: bug kind).

### note · agent:pm-1 · 2026-11-21T16:18:00.940Z
pm-1 brief: model Sonnet. Same area as the existing check: crates/bridle-daemon/src/server.rs (`require_not_worker` ~828-848, the ready gate ~3447-3461), the role config in crates/bridle-daemon/src/config.rs, docs/design/agent-host/roles-and-config.md.
- Decide by what the role is. Read how roles are configured first (a role with a worktree and a "worker" base, e.g. researcher in .bridle/config.toml) and use the property that already exists if there is one; add a new config key only if nothing says "this role implements a ticket in a worktree". Fail closed: an unknown or custom role gets no lifecycle authority unless it is explicitly one of the managing roles (manager, orchestrator, project-manager where it is allowed today) or a non-agent principal. Say in one line why that default.
- Same test for the lifecycle endpoints and the ready gate (if the ready gate means the same thing, as the ticket says; otherwise leave it and say so).
- Migration: if a config key is added, existing projects must keep working with no edit: define the default so the roles a project has today (worker, researcher, designer, prototyper, document-reviewer, manager, orchestrator, project-manager, advisor, aide) keep or lose authority exactly as the ticket intends; if a project's own custom role would change behaviour, `bridle daemon doctor` or a log line should say so. No project-file rewrite expected.
- Tests: researcher-style custom role gets 403 on spawn, stop and remove; manager and orchestrator still succeed; worker still refused; ready gate per the ticket. `just check` with BRIDLE_* env unset. Out of scope: tool deny-lists per role.
