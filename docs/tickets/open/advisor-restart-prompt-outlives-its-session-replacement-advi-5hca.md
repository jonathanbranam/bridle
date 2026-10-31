---
id: br-5hca
title: "Advisor restart prompt outlives its session: replacement advisor obeys it and restarts again"
kind: bug
created: 2026-10-31
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Reported by orchestrator@remus, relaying advisor glassbox@remus (the human noticed it), message m-0513 on dalek-home.

What happened (glassbox, 2026-10-31 UTC): advisor session pid 3837706 wrote handover h-0029 at 03:23:42.086. 46 ms later the "The human is restarting this session. Write the handover note..." prompt went out as inbox message m-2375 (note, external:advisor -> external:advisor). The session ended at 03:23:42.714 without reading it. Advisor inboxes are keyed by identity, not session, so the replacement session (144988) got m-2375 on its first wait, obeyed it, wrote h-0030 and was restarted too. The next session (147083) was clean. Evidence: glassbox daemon events seq 15881-15888, message m-2375.

Two problems: (1) the restart prompt is sent after the handover is already recorded; (2) it is addressed to the shared identity, so it outlives the session it was meant for.

Fix: do not send the restart prompt when the session has already written its handover, and scope the note to the target session (or expire/drop it when that session ends) so a replacement never receives its predecessor's prompt. Find where the restart command queues that note (daemon session restart/handover code, docs/design/agent-host/).

Acceptance: a test where a session restarts before reading the restart note: the replacement session's first wait does not deliver it; and no prompt is sent after a handover is already recorded. `just check`. Model: Sonnet. Out of scope: how advisor inboxes are keyed in general.
