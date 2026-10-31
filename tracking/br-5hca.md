+++
id = "br-5hca"
title = "Advisor restart prompt outlives its session: replacement advisor obeys it and restarts again"
kind = "bug"
state = "pending"
created_at = "2026-10-31T03:27:41.121Z"
updated_at = "2026-10-31T03:27:48.192539Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "5hca"
+++

docs/tickets/open/advisor-restart-prompt-outlives-its-session-replacement-advi-5hca.md

submitted by external:orchestrator

Reported by orchestrator@remus, relaying advisor glassbox@remus (the human noticed it), message m-0513 on dalek-home.

## What happened (glassbox, 2026-10-31 UTC)

- Advisor session pid 3837706 wrote handover h-0029 at 03:23:42.086.
- 46 ms later the "The human is restarting this session. Write the handover note..." prompt went out as inbox message m-2375 (note, external:advisor -> external:advisor).
- The session ended at 03:23:42.714 without reading it.
- Advisor inboxes are keyed by identity, not session, so the replacement session (144988) got m-2375 on its first wait (03:23:52), obeyed it, wrote h-0030 and was restarted too (03:24:11).
- The next session (147083) came up clean.

## Two problems

1. The restart prompt is sent after the handover is already recorded (ordering).
2. It is addressed to the shared identity, so it outlives the session it was meant for.

## The ask

Scope the restart note to the target session, or drop/expire it when that session ends, so a replacement session never receives its predecessor's restart prompt. Also don't send the prompt when the session has already written its handover.

## Evidence

glassbox daemon events seq 15881-15888, message m-2375.

## Verify

A test where a session restarts before reading the restart note: the replacement session's first wait does not deliver it.

## Thread

### note · external:orchestrator · 2026-10-31T03:27:41.121Z
submitted by external:orchestrator
