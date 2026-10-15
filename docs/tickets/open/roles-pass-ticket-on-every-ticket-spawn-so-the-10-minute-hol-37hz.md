---
id: br-37hz
title: Roles pass --ticket on every ticket spawn so the 10-minute hold applies (follow-up to 85hs)
kind: fix
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The 10-minute hold (br-85hs, landed) only covers spawns that pass `--ticket`. The manager role spawns with `bridle agent spawn worker --name <short-name> --prompt "<task>"` (workflow/base/roles/manager.md:36), no --ticket, so the case that prompted 85hs (a designer on br-dffz started seconds after the request) is still unheld.

Wanted: every role instruction that spawns an agent for a ticket passes `--ticket <id>`. Chosen over having the daemon infer the ticket from --name, which is a fragile guess (names are short, not always ids).
