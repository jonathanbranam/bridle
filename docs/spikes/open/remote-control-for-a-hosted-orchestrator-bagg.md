---
id: bagg
title: Spike: can Remote Control reach an orchestrator bridle supervises?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

- Whether a Remote Control session can attach to, or be started for, a
  `claude` process that bridle spawned. Bridle-hosted agents are headless
  `claude -p` stream-json.
- Whether a systemd-supervised `claude remote-control` (as in the NUC guide)
  keeps one orchestrator conversation across restarts, or starts fresh each
  time.
- Whether a session started under Remote Control can be resumed by id with
  `--resume`, so bridle could know about it.

## Why it matters

It decides whether bridle can host the one orchestrator the human reaches from
laptop and phone: [[where-the-single-orchestrator-lives-hj4g|where the orchestrator lives]].

## Notes

Raised on 2026-09-27 while checking the NUC plan ([[docs/context/nuc-host|NUC host]]) against the agent-host design. None of this has been tested.
