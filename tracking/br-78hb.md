+++
id = "br-78hb"
title = "Bug: worker-like roles (researcher, designer, prototyper, document-reviewer) pass the lifecycle check, which matches the role name worker exactly"
kind = "bug"
state = "built"
created_at = "2026-11-21T16:17:17.032Z"
updated_at = "2026-11-21T17:37:48.324271Z"
created_by = "external:advisor/agentic-dev"
watchers = [
    "external:advisor/agentic-dev",
    "agent:w78hb",
]
branch = "bridle/w78hb"
commit = "e1083d9c729f90027c74237a7eb17f2ccbac4f7c"
summary = "require_not_worker and the ready gate now ask the role's config (workdir = worktree, via AgentManager::role_is_worktree) instead of the name 'worker'; a role missing from config counts as worker-like (fail closed). No new config key, no migration: researcher, designer, prototyper, document-reviewer and custom roles keeping the default are now refused; manager, orchestrator, advisor, aide, project-manager (workdir repo) unchanged. A custom role with workdir=repo still passes, as before. Test: worker_like_roles_are_refused_agent_lifecycle_endpoints (researcher and a custom worktree role get 403 on spawn, stop, remove). Docs: principals.md. No doctor/log line added (no role loses or gains authority except the worker-like ones the ticket targets)."
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

### note · agent:w78hb · 2026-11-21T17:37:40.970Z
built: e1083d9c729f90027c74237a7eb17f2ccbac4f7c (branch bridle/w78hb)

### note · agent:w78hb · 2026-11-21T17:37:43.184Z
done: lifecycle endpoints + ready gate refuse every workdir=worktree role (unknown role fails closed); just check exit 0, 2023 tests passed; e1083d9c729f90027c74237a7eb17f2ccbac4f7c

### note · agent:manager-2 · 2026-11-21T17:37:48.324Z
Diff looks good. main moved (br-nkgf landed). Please merge main into your branch, rerun env -i HOME=$HOME PATH=$PATH just check, commit, and message me the new sha, exit status and test count.
