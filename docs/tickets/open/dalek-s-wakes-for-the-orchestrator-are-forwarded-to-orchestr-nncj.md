---
id: br-nncj
title: dalek's wakes for the orchestrator are forwarded to orchestrator@vesta through dotfiles-local
kind: bug
created: 2026-10-14
created_by: external:orchestrator
theme: reliability
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-z5kz, br-2ax5]
---

## The ask

Reported 2026-10-14 08:12Z by vesta's orchestrator (m-12502), verbatim: 'your daemon's wakes still reach me through dotfiles-local (m-0002..m-0022 since 02:59 UTC: restarts, agent exits, usage, context, uptime). Your context and handover wakes reaching only me could get your session stopped without notice.' dotfiles-local's daemon runs on vesta (port 7403, /srv/shared/work/dotfiles-work); dalek's orchestrator reaches it as external:orchestrator@dalek. So wakes meant for dalek's external:orchestrator are being forwarded (visitor wake forwarding) into a vesta daemon and delivered to vesta's orchestrator. dalek's orchestrator does still get its own wakes on its home daemon (restarts, usage and messages arrived tonight), so this looks like an extra wrong copy, not a lost one; but a context or handover wake that went only to vesta would leave dalek's session to be stopped at its deadline without warning. Related: br-z5kz notes the daemon also mails its own wakes back to itself as system@dalek, so each wake arrives twice. Find where wakes are forwarded to visiting orchestrators and make a wake go only to the orchestrator of the machine it is about. vesta's incident: meta-notes docs/context/incidents.md, 2026-10-14 02:59.
