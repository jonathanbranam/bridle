---
id: br-78hb
title: "Bug: worker-like roles (researcher, designer, prototyper, document-reviewer) pass the lifecycle check, which matches the role name worker exactly"
kind: bug
created: 2026-11-21
created_by: external:advisor/agentic-dev
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-11-21T17:42:38Z
---

Found by advisor agentic-dev while checking the notes project's agentic-dev survey against bridle main 153d3c86.

`require_not_worker` (crates/bridle-daemon/src/server.rs:828-848) refuses agent lifecycle endpoints (spawn, interrupt, stop, resume, renew, remove) only when the calling agent's stored role is exactly `worker`:

```
if role.as_deref() == Some("worker") {
```

Its doc comment and docs/design/agent-host/roles-and-config.md say worker-role agents get no lifecycle authority. But roles that are workers in all but name pass the check: `researcher` (.bridle/config.toml, "A worker that can also search and fetch from the web", br-rz4e), and the base roles `designer`, `prototyper` and `document-reviewer` (workflow/base/roles/). Any of them can spawn, stop or remove agents through the API if a tool deny-list doesn't stop the Bash command first. The deny-list is a second line of defence that differs per role, not the check itself.

The ready gate has the same shape (server.rs:3447-3461): a deny-list of `worker` and `project-manager` by name, so other worker-like roles may ready tickets.

## The ask

Make the lifecycle check (and the ready gate, if it means the same thing) decide by what the role is, not by its literal name: refuse every role that is worker-like (spawned with a worktree to implement a ticket, i.e. the built-in worker and any role configured from it), and allow only the roles that are meant to manage agents (manager, orchestrator, and non-agent principals as today). Pick the property in config that says so (or add one), and use the same test in both places. Add a test that a `researcher`-style custom role gets 403 on spawn. Update roles-and-config.md to say how the rule is decided.
