---
id: br-eupm
title: Orchestrator replies go to the bare role on its own home daemon, not to the sender, so the sender never reads them
kind: bug
created: 2026-10-25
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: [br-q7my]
related: [br-q7my]
---

## The ask

The human, 2026-10-25 ~11:30 AM ET (to aide), verbatim: "orch are running but they don't seem to be addressing their replies to the sender correctly".

What aide saw (2026-10-25): on vesta-home's daemon, the orchestrator sent two messages to bare `external:aide`, m-0113 (14:57Z) and m-0118 (15:05Z), both with no `reply_to`, and both still undelivered and unread. They came right after aides wrote to it: vesta-home m-0112 (`external:aide@vesta`, 14:56Z), m-0115 and m-0116 (`external:aide`, 14:58Z and 15:04Z). An aide listens only on its own project's daemon (role `aide`, "Identity"), and no aide waits on vesta-home, so a reply sent to `external:aide` there is never read. Identities are shared too (every project's aide is `external:aide`), so even the right daemon can't tell which aide is meant from the bare role. Compare the meta-notes aide, which had to write "Reply to external:aide on meta-notes" into its message body to get an answer back.

Fix: a reply reaches the principal that sent the message, on the daemon it sent from. Pick the mechanism: `bridle send --reply-to <id>` routes to the original sender's project and identity (the `@host`/project kept on the message), and/or the orchestrator role and `bridle prime orchestrator` tell it to always reply with `--reply-to`. A reply that can't be routed fails loudly rather than sitting in an inbox nobody reads.

Acceptance: a message from an aide on project P to `external:orchestrator --project <host>-home`, answered with `--reply-to`, arrives in that aide's wake on P (test across two daemons, or a unit test of the routing). The orchestrator role doc says how to reply. Docs: docs/design/agent-host/ (messages), workflow/base/roles/orchestrator.md. `just check`.
