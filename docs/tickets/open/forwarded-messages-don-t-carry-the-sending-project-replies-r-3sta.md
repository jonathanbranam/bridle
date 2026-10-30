---
id: br-3sta
title: Forwarded messages don't carry the sending project; replies route to the receiving machine's default project
kind: bug
created: 2026-10-30
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-jpph]
---

## The ask

The human asked for this (2026-10-30, via orchestrator@remus m-0503, in orch-remus-home; filed by orchestrator@dalek): "yes", to filing a ticket so forwarded messages carry the sending project.

When mail is forwarded between daemons, a plain reply to the forwarded address (for example `external:aide@vesta`) routes to that machine's default project (`dotfiles-local` on vesta), not the project the sender wrote from. Workaround today: `bridle send --project <project> external:<role> ...`.

Fix: a forwarded message carries the sending project (the outbox forward payload and the receiving daemon's forwarded_in record), and a reply to it routes back to that project by default, including bare `bridle send --reply-to <id>` (see br-jpph and br-eupm for the reply_route / forwarded_origin code; read what they landed first, as this extends the same path and may already be partly fixed).

Migration: the forward payload gains a field; an older peer that omits it keeps today's behaviour (default project), and a newer daemon must accept payloads without it. Say so in docs/design/agent-host/ (messages) and the CHANGELOG.

Acceptance: a two-daemon test where a message from project P on daemon A, forwarded to daemon B, is answered by a plain reply and arrives on P; a payload without the field still delivers to the default project; `just check`. Model: Sonnet. Out of scope: --task on cross-daemon replies.
