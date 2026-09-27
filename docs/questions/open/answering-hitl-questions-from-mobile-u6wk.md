---
id: u6wk
title: Answering HITL questions from mobile
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [k4wq, hj4g]
---

## What happened

The human, verbatim:

> there should only be one orchestrator, but I might have the NUC bridle start one and
> turn on remote-control so I can talk to it, or I might run the orchestrator locally. If
> that doesn't conflict, anyway, TBD, but sometimes I move from having laptop access to
> mobile-only but I still want to interact with the work and address any HITL questions
> that arise or provide aditional ideas and thoughts I have while remote.

## Why it matters

Questions block tasks until answered
([[docs/design/coordination#Questions do not stop work|questions]]). A human who is
mobile-only for a day can't unblock anything if they don't know a question
is waiting.

## Notes

Raised on 2026-09-27 while checking the NUC plan ([[docs/context/nuc-host|NUC host]]) against the agent-host design.

- **No notification.** Nothing tells the human that a question has arrived.
  Something has to watch the event stream for questions addressed to `human`
  and push a notification (e.g. ntfy).
- **Provenance.** An answer relayed through the orchestrator is recorded as
  `external:orchestrator`, not `human`. Either accept that, or add a
  "relayed for human" field.
- **A small web inbox** served by the daemon, opened from the phone over
  Tailscale, avoids public exposure.
- **MCP from claude.ai mobile** ([[docs/proposal/build-order|build order]]) needs bridle
  on public HTTPS, e.g. Tailscale Funnel. claude.ai's connectors run on
  Anthropic's servers, not on the phone, so they can't reach the tailnet.

The requirements here are to be refined later. The human's tentative preference (not yet a
spec): a claude.ai connector reachable only via public HTTPS to `/mcp`, with OAuth,
exposing a small voice-shaped tool set — briefing, pending questions, answer question, tell
manager, budget hold/release, approve held merges — deliberately **not** spawn/stop/rm or
anything else that could run arbitrary work from a phone call.
