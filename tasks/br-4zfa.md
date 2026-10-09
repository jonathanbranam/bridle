+++
id = "br-4zfa"
title = "Incident: the bridle orchestrator was killed (SIGTERM) at 11:18 PM ET and nothing relaunched it overnight"
kind = "incident"
state = "pending"
created_at = "2026-10-06T12:05:50.523Z"
updated_at = "2026-10-09T11:04:36.180951Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
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

### note · external:orchestrator · 2026-10-06T12:08:26.238Z
Root cause, found by the orchestrator (resumed session, 2026-10-06 ~8:10 AM ET):

1. WHO SENT SIGTERM: the daemon, on purpose. Event 03:19:05Z: 'The orchestrator was stopped at 03:19:05 UTC: the handover deadline passed with no bridle handover done.' The session had passed 200K and got 'hand over now ... stopped at 03:18:54 UTC'. It wrote handover h-0049 but never ran 'bridle orchestrator handover done', the step its prime names twice. The stop was the designed fallback.

2. WHY NO RELAUNCH: event 03:19:05Z: 'The orchestrator is down and no pane is tagged @bridle=orchestrator'. The orchestrator's own pane (%68) was tagged @bridle=advisor-fields. At ~9:10 PM ET the orchestrator ran 'bridle session advisor fields ...' from its Bash tool (incident br-9z2d). session.rs:378 calls pane::tag_pane, which tags $TMUX_PANE, and that is the calling Claude's pane, so it RETAGGED THE ORCHESTRATOR'S PANE as advisor-fields. That run ended in print mode, but the tag stayed. From then on the supervisor had no pane to relaunch into.

3. WHY NOBODY KNEW: both facts were raised only as orchestrator.incident events (03:19:05Z, also 03:03:57Z 'no wake command running'), and they reached neither the aide nor the human overnight.

Fixed now: pane %68 retagged @bridle=orchestrator (bridle pane tag orchestrator).

Recommendations for the postmortem:
(a) session commands must not tag the pane when they aren't interactive (no TTY), or must refuse to run without a TTY (fne2);
(b) relaunch with no tagged pane should fall back to a new tmux window, not give up;
(c) 'orchestrator down' must reach the human (aide, push notification, the human's to-do list), not only the event log;
(d) the orchestrator must run 'handover done' (role text is there; the session didn't read its prime, incident 9z2d), or the daemon should do the stop and relaunch without it.

### note · external:orchestrator · 2026-10-06T22:52:27.054Z
Follow-up (2026-10-06 6:35 PM ET): after the human resumed the orchestrator by hand ('claude --resume', not the launcher), the daemon never adopted the session. There's no session row, no BRIDLE_AS (every bridle command failed until prefixed), and no context wakes. It records 'orchestrator is down but pane %68 is running 2.1.291, not a shell; not typing into it', so it also won't relaunch while this session lives. A hand-resumed session is invisible to the supervisor and can't be restarted by it. Fix ideas: let 'bridle session orchestrator --resume <id>' register a resumed session, or adopt a claude running in the tagged pane.

### note · external:advisor/product-manager · 2026-10-09T11:04:36.180Z
watching the task
