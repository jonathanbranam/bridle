---
id: hj4g
title: Where does the single orchestrator live?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [u6wk, xqvg]
---

## What happened

The human, verbatim:

> there should only be one orchestrator, but I might have the NUC bridle start one and
> turn on remote-control so I can talk to it, or I might run the orchestrator locally. If
> that doesn't conflict, anyway, TBD, but sometimes I move from having laptop access to
> mobile-only but I still want to interact with the work and address any HITL questions
> that arise or provide aditional ideas and thoughts I have while remote.

## Why it matters

The orchestrator is the human's interface to the workforce. If it lives on the
laptop, it goes away when the laptop sleeps, which is the moment the human
switches to mobile.

## Notes

**Needs** spike `bagg` first: [[remote-control-for-a-hosted-orchestrator-bagg|remote control for a hosted orchestrator]].

Raised on 2026-09-27 while checking the NUC plan ([[docs/context/nuc-host|NUC host]]) against the agent-host design.

- [[docs/design/agent-host/operating-model|The operating model]] makes the orchestrator optional and external, with its
  own `external:orchestrator` token. It can run anywhere, or bridle can host
  one (`bridle spawn orchestrator`).
- A bridle-hosted agent is headless `claude -p` stream-json. It isn't a Remote
  Control session, so "the NUC bridle starts one and turns on remote-control"
  isn't possible as designed. That's untested:
  [[remote-control-for-a-hosted-orchestrator-bagg|remote control for a hosted orchestrator]].
- One option that fits the design as it stands: run the orchestrator on the NUC,
  outside bridle, under systemd (like the NUC guide's `claude-rc` unit). Laptop
  and phone both reach the same session through Remote Control, so there is
  only ever one.
- Whichever machine hosts it, the orchestrator's lasting state should be in
  bridle (inbox, events, later tasks), not in its context. Then a fresh session,
  or a move between machines, picks up where the last one stopped.
- Nothing in the design enforces a single orchestrator. Two sessions with
  `external` tokens could both drive a daemon.
