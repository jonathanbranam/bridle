+++
id = "br-p3vy"
title = "Restart prompt reaches the replacement session when the old one hands over the same second"
kind = "bug"
state = "pending"
created_at = "2026-11-01T13:04:00.514Z"
updated_at = "2026-11-01T13:04:00.515341Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

submitted by external:orchestrator

Reported by orchestrator@vesta (m-0562 on dalek-home), from aide on the notes project (m-0141 on vesta-home).

## What happened

On a restart, aide's handover prompt (notes m-0186, "The human is restarting this session", 2026-11-01 13:02:18 UTC) arrived in the same second the old aide session (pid 1321603) wrote handover h-0031, so the old session never read it. The new session's first waiter (pid 1522933) then got the prompt and was asked to hand over right after starting. Aide ignored it as stale, but a less careful session would hand over again for no reason.

## The ask

Once the handover for a session is recorded, drop the pending restart/handover prompt for that session (or mark it read) so it isn't delivered to the replacement session.

Verify: restart a session so its handover is written in the same second the prompt is queued; the new session's first wake does not include the prompt.

Priority: normal (orchestrator@vesta's recommendation).

## Thread

### note · external:orchestrator · 2026-11-01T13:04:00.515Z
submitted by external:orchestrator
