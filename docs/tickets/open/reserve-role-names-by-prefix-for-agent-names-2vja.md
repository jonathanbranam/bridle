---
id: 2vja
title: Reserve role names, by prefix, for agent names
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [3ehu]
---

## The ask


The human, 2026-10-01, verbatim (via the advisor):

> Agent names for specific roles should be reserved and reserved by prefix so that I know
> orchestrator is that role and advisor-Jim is definitely an advisor. Create a ticket to reserve
> roles and limit who can create agents from those reserved names.
>
> Any name without a reserved base or prefix is allowed unless something else restricts it.

## How it came up

br-a911 (8939ca0, 3ehu part 2) made `bridle send advisor` answer "did you mean
external:advisor?". The daemon looks the recipient up as an agent name first
(`crates/bridle-daemon/src/server.rs`) and offers the hint only when no agent has that name. So a
local agent named `advisor` or `orchestrator` would silently receive messages meant for the
external principal. The human asked who can create agents.

## What's there now

- Spawn is refused only for `agent` principals whose role is `worker`. Managers, the
  orchestrator, the human and other external principals can spawn under any name
  (`docs/design/agent-host/principals.md`, the agent lifecycle paragraph, at ada1d61).
  Managers name the workers they spawn after the task (`send-hint`, `deny-write`).
- Role names in `workflow/base/roles/`: advisor, manager, orchestrator, product-manager, worker. Named advisors already use the
  `advisor-<name>` form, as the tmux tag (`@bridle` = `advisor` or `advisor-<name>`,
  `bridle_daemon::focus`) and in `bridle session advisor <name>`.
