---
id: qun8
title: The roles a project needs start with the daemon by default
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
closed: 2026-09-30T05:12:44Z
---

## The ask

The human, verbatim (2026-09-28), on meta-notes' first daemon start: "the manager didn't
startup automatically, that should happen by default; the roles we need should just
start".

## Notes

- `autostart` defaults to `false` for every role, the built-in manager included
  (`docs/design/agent-host/roles-and-config.md`); a role with `autostart = true` gets an
  agent named after it at daemon start (`docs/design/agent-host/daemon.md`, startup).
- meta-notes: the daemon first started at 22:40 UTC with no manager; `autostart = true`
  was added to its `[roles.manager]` at 22:41 (meta-notes af8b6c0 on `bridle-adopt`), and
  the restart at 22:52 spawned `manager`.
- bridle's own `.bridle/config.toml` sets no `autostart` on `manager` or
  `product-manager`; they persist through `resume_on_restart`.

## Resolution

The built-in `manager` autostarts by default (`docs/design/agent-host/roles-and-config.md`,
`daemon.md` startup; covered by `manager_autostart_test.rs`: default starts one, explicit
`autostart = false` disables, a restart doesn't duplicate). `product-manager` is not a built-in
role but bridle's own custom one, so it opts in with `autostart = true` in `.bridle/config.toml`.
Autostart uses the ordinary spawn, so a budget hold refuses it.

Resolved 2026-09-29.
