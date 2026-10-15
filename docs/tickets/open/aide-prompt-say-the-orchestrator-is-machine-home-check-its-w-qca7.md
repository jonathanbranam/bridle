---
id: br-qca7
title: "aide prompt: say the orchestrator is <machine>-home, check its waiter there, and listen only on your own daemon"
kind: bug
created: 2026-10-15
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask


The aide role prompt (workflow/base/roles/aide.md) doesn't say there is one orchestrator per machine. Filed by the data-contracts aide on remus at the human's request (2026-10-14 ~9:05 PM ET).

What happened: primed in data-contracts, the aide checked waiter_open on the data-contracts daemon (false) and told the human no orchestrator was watching. The orchestrator lives on remus-home. After sending to external:orchestrator --project remus-home it replied, and the reply arrived on the aide's own daemon.

Wanted in aide.md:
1. One orchestrator per machine: the <machine>-home project (e.g. remus-home). Today only a passing mention in the relay bullet.
2. "Relaying to the orchestrator" checks waiter_open on that daemon: bridle status --json --project <machine>-home.
3. The aide listens only on its own project's daemon; replies from the orchestrator and other daemons are forwarded there automatically (mail is auto-forwarded).
