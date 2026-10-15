---
id: br-mfcj
title: Mail bridge hangs for good after one stalled S3 download (minimum throughput 0 B/s)
kind: bug
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-bc37]
related: []
---

## The ask

Reported by meta-notes' aide on vesta, 2026-10-15, after the human's email to dotfiles-local did not arrive.

On 2026-10-14 ~17:07 UTC, during a network outage on vesta, three of four mail bridges (meta-notes, meta-notes-ui, dotfiles-local) each logged one warning, "streaming error: minimum throughput was specified at 1 B/s, but throughput of 0 B/s was observed", and then nothing for 29 hours. The processes stayed active (running) with an ESTAB connection to S3 but stopped polling: no credential reloads, digests or deliveries. systemd Restart=on-failure never fired because nothing exited. The human's email to dotfiles-local@ sat in inbound/ until the aide restarted the three bridges.

Ask:
- A stalled or failed S3 read must not wedge the poll loop: time out the whole GET and list, and carry on to the next poll.
- A watchdog: when the last successful poll is older than N x poll_secs, raise an incident on the project's daemon (or exit nonzero so systemd restarts it). A silent bridge looks the same as no mail.

