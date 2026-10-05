+++
id = "br-cbbn"
title = "Scheduled nightly restart of an interactive session at a clock time (e.g. 3 AM)"
kind = "feature"
state = "pending"
created_at = "2026-10-05T10:25:03.871Z"
updated_at = "2026-10-05T10:25:03.871817Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

submitted by external:orchestrator@nuc

The human, 2026-10-05: 'Whatever agent I use to take all of my notes, I want it to just restart in the middle of the night. I think it feels a little risky to trust that system to work and the handover to work. ... maybe 3 am is better so that it's always restarted before the morning.' and 'I think the orchestrator on Dalek has a rule that it gets refreshed every 12 hours. I don't want to add that rule to other agents unnecessarily.'

Today: max_uptime (12h) applies to the orchestrator only; gq9r covers context-driven handover for sessions, not a clock time. Ask: per-session (or per-role, per-project) config such as restart_at = "03:00" (human's time zone): at that time the daemon asks the session for its handover (with the role's handover instructions, see the companion ticket), waits for it (with a deadline), restarts the session in its pane, and the new session opens with the note. Opt-in, so other agents are unaffected. First user: the notes project's advisor (the human may switch notes to an aide role). Should skip or defer if the human is mid-conversation at that moment (last_activity within N minutes).

## Thread

### note · external:orchestrator@nuc · 2026-10-05T10:25:03.871Z
submitted by external:orchestrator@nuc
