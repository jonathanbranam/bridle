---
id: 9t54
title: Reload config without restarting the daemon
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [project-machine-and-account-scope-9mxw]
closed: 2026-10-02T00:43:35.989404Z
---

## The ask

The human (2026-09-29): "it would be nice to refresh config with a SIGHUP or IDK, some way to
do that without a restart, but that is probably a much bigger job."

Today every change to `.bridle/config.toml` or `~/.bridle/config.toml` (thresholds, roles,
`[orchestrator]`, `[context]`) needs `bridle stop-daemon` and `bridle serve`, which drops every
running agent to `lost` and needs a resume by hand. Config changes then wait for a maintenance
window alongside rebuilds.

## To decide

- The trigger: SIGHUP, a `bridle reload` command (an API endpoint), a file watch, or several.
- Which sections can change live (thresholds, budget, context, orchestrator: likely easy, they're
  read on each tick) and which can't without more work (roles already spawned keep their
  launch settings until renewed or resumed; `[daemon]` bind settings).
- Validation: a bad file keeps the old config and reports why, never half-applies.
- Where the answer to 9mxw (which sections are machine vs project) changes what reloads from where.

Not urgent: restarts are cheap while agents resume cleanly.

## Update (2026-09-30)

The human asked again: "config changes should be reloaded more often." Focus hours (cvaq) already
re-read `[[focus]]` on every prompt; everything else is read only at daemon start. Self-restart
(q7rx) makes a restart cheap but still waits for a quiet point, so it isn't a substitute for
settings that are safe to read per tick. Start with the sections that are read on each tick.

## Resolution

Resolved by: br-a618 (40f26fc)
