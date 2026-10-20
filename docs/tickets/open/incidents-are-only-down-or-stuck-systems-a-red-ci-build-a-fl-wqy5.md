---
id: br-wqy5
title: "Incidents are only down or stuck systems: a red CI build, a flaky test or a bug is a bug ticket, not an incident"
kind: chore
created: 2026-10-20
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Narrow what counts as an incident, in the rules and docs that tell roles what to log and file.

The human, 2026-10-20 ~6:50 PM ET (to aide), verbatim: "I think we're logging too much as
incidents. Red build on CI is not an incident. incidents are down or blocked systems that aren't
moving. not bugs or flaky tests"

- **An incident** is a system that is down, or blocked and not moving: a daemon that can't
  restart, an orchestrator that died and nothing relaunched it, messages held for hours, spawns
  blocked by load.
- **Not an incident:** a red CI build, a flaky test, a bug. Those are `bug` (or `fix`) tickets,
  handled the usual way. Neither an `incident` ticket nor an entry in `docs/context/incidents.md`.
  (If a red main blocks all landing for a long time, *that* stall can be an incident.)

Places that say otherwise today:
- `workflow/base/roles/orchestrator.md` "Keep the incident log": lists "a red `main`" and asks to
  log "every failure or problem".
- `docs/context/incidents.md` header: same list ("a red `main`, a bad merge ...").
- `docs/design/agent-host/incidents.md`: check that its definition of an incident matches.
- Any other role or rule that tells a role to file or log CI failures or flakes as incidents
  (manager, project-manager).

Existing entries and incident tickets stay as they are; this is about what is filed from now on.
