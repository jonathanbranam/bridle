---
id: 2fkk
title: Do workers resume after a daemon restart?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [t39j]
---

## The question

From `docs/agent-host.md` §4.7:

> Agents are the daemon's children through pipes. **If the daemon dies, each
> agent's stdin reaches EOF and the agent exits after its current turn.** …
> Agents whose role has `autostart = true`, or that were `lost` with
> `resume_on_restart` (the default for the manager), are resumed with
> `--resume`.

Workers default to `resume_on_restart = false`, so after a restart they stay
`lost` until someone runs `bridle resume`.

## Why it matters

Every daemon restart (an upgrade, a crash, a nightly reboot:
[[agents-and-nightly-auto-reboots-t39j|nightly reboots]]) leaves workers
stopped mid-task. Either the manager notices and resumes them, or the human
does. With the task store (P0), a lost worker's claim might be released
instead of resumed.

## Notes

- Options:
  - resume every `lost` agent;
  - resume only agents whose last turn ended cleanly;
  - let the manager decide (it gets `agent.state` events);
  - tie it to claims once tasks exist.
- Related: whether agents should survive a daemon restart at all, e.g. by
  giving the daemon's stdio pipes to a small per-agent holder process. §4.7
  rules this out for v1.
