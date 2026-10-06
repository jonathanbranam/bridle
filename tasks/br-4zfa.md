+++
id = "br-4zfa"
title = "Incident: the bridle orchestrator was killed (SIGTERM) at 11:18 PM ET and nothing relaunched it overnight"
kind = "incident"
state = "pending"
created_at = "2026-10-06T12:05:50.523Z"
updated_at = "2026-10-06T12:05:50.523960Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
priority_at = "2026-10-06T12:05:50.523960Z"
+++

The human, 2026-10-06 morning (verbatim): "I dont see a bridle orchestrator running - log an incident. Im actually going to directly resume that claude session since Im here at home so it can also check."

Facts from the human screen grab of the orchestrator pane (session orch-bridle):
- Last turn done 11:05 PM ET (2026-10-06 03:05Z): it re-armed `bridle orchestrator wait-for-wake --timeout 1500` (3 shells still running), handover h-0049, ctx 21% (206k).
- The launcher logged: `2026-10-06T03:18:56Z 11:18:56 PM -04:00 orchestrator claude ended: exit 143 (SIGTERM)`, about 14 minutes after its last turn.
- Nothing relaunched it overnight: the pane was at a shell prompt in the morning, and no relaunch or incident reached the aide.
- The Claude Code footer showed "Update installed - Restart to update".
- The input line held `bridle gateway --detach`, typed but not sent.

To find out: who sent SIGTERM at 03:18:56Z (cf. fx7x, pkill by pattern; the no-kill-by-name rule), and why the supervisor did not relaunch it (relaunch, backoff, held relaunch 8fsx, tmux pane tag).
The human is resuming the session by hand (`claude --resume orch-bridle`) so it can check too.

## Thread

### note · external:aide · 2026-10-06T12:05:50.523Z
priority: normal -> high
