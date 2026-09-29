---
id: nc7r
title: Incident notices that go to every agent and disappear when it's over
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [kp3f, ex9q]
---

## The ask

The human, verbatim (2026-09-29):

> When something big happens like SSH is down, messages should go out to agents and then
> updated when the incident is over. Ideally, the message should be removed. This kind of
> need has come up repeatedly and should be considered.
>
> Eg advisor don't know ssh was down. But let's say you sent a message when advisor was down
> then we fixed ssh then advisor came up. Ideally, advisor would never know about the incident.

## Why

On 2026-09-29, after a reboot, the SSH agent had no key and `git push` failed. The
orchestrator told manager-2 by hand; the advisor, pm-1 and the other projects' managers
never heard, and the advisor found out on its own. Earlier cases of the same shape: the
daemon restarting (every `lost` worker told by hand), budget holds, a red `main` ("don't
merge until green"), the syspolicyd hangs (`docs/context/incidents.md`).

## Sketch (not a design)

An **incident** (or notice) is a record, not a message: opened with a short body and an
audience (all agents, a role, external principals, other projects' daemons?), and closed
when it's over.

- While open, every agent in the audience sees it: delivered to running agents, and to an
  agent that starts or resumes while it's open (in its first message, or the system prompt).
- Closing it sends a short "resolved" to those who saw it. **An agent that never saw it
  never hears of it**, and an undelivered notice is dropped, not delivered late.
- Updates while open replace the body (agents who saw it get the update).
- `bridle status` lists open incidents; the watcher could wake on one opening.

Open: who opens one (the orchestrator, managers, the daemon itself for things it detects,
like a failed push, a red `main` via c8qw, or a budget hold); whether it spans projects
(one incident, three daemons); whether the budget governor's hold notices become incidents.
Held messages (`when`/pending delivery) may be most of the mechanism already: a message
that is withdrawn while still pending is never delivered.
