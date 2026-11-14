+++
id = "br-t8wk"
title = "Aide and advisor sessions get no waiter-grace check: an aide sat with no waiter for ~50h unnoticed"
kind = "bug"
state = "planned"
created_at = "2026-11-14T11:06:11.413Z"
updated_at = "2026-11-14T12:00:14.311924Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "t8wk"
+++

docs/tickets/open/aide-and-advisor-sessions-get-no-waiter-grace-check-an-aide-t8wk.md

submitted by external:orchestrator

Filed by orchestrator@dalek for the human, relayed by orchestrator@vesta (m-0652 on dalek-home): "the human asked for a bridle ticket or incident".

## What happened

On vesta a DNS outage (2026-11-12 07:41 to at least 11:12 UTC) made the meta-notes aide's API call fail at 08:34:42 with `API Error: Can't reach the API server (EAI_AGAIN)`. Claude Code ended the turn, and the aide's waiter (`bridle agent wake external:aide`) was not running. Nothing restarted it and nothing alerted on it for about 50 hours, until the human typed "Try again" at 2026-11-14 11:00 UTC.

`bridle status` showed "no waiter" for the aide the whole time, but waiter_grace / the waiter incident covers only the orchestrator.

## The ask

- Give aide (and advisor) sessions the same waiter-grace check as the orchestrator: after the grace period with no waiter, record an incident and send a note to the orchestrator.
- Consider typing a nudge into the tagged tmux pane (`@bridle=aide`) when the session is idle at a shell-safe prompt with no waiter.

## Evidence

- meta-notes `docs/context/incidents.md`, entry 2026-11-12 08:34 (commit cd12073).
- vesta transcript `~/.claude/projects/-srv-shared-work-meta-notes-work-meta-notes/0a9e2954-....jsonl`, line 1561.

## Thread

### note · external:orchestrator · 2026-11-14T11:06:11.421Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-11-14T11:06:57.737Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-14T11:11:15.967Z
Cause of the outage, human via orchestrator@vesta: a UPS failure shut off the home router running DNS, so the whole home had no internet. Other sessions recovered on their own; only the meta-notes aide stayed stuck with no waiter.

### note · external:orchestrator · 2026-11-14T11:49:58.717Z
Correction, human via orchestrator@vesta: the human did not type 'Try again'. They opened Remote Control, which showed 'cannot connect' with a 'Try again' button; clicking it sent the literal text 'Try again' to the aide as a prompt, and that revived it. So the session recovered only by chance, when the human happened to open it. (Same pattern seen on dalek: this orchestrator's session also received a bare 'Try again' prompt on 2026-11-14.)
