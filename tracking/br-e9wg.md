+++
id = "br-e9wg"
title = "Automatic push alarms on a one-off race, and its message hides git's reason"
kind = "bug"
state = "open"
created_at = "2026-10-11T14:29:42.746Z"
updated_at = "2026-10-11T17:17:54.888343Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "e9wg"
+++

docs/tickets/open/automatic-push-alarms-on-a-one-off-race-and-its-message-hide-e9wg.md

## Thread

### note · external:orchestrator · 2026-10-11T16:16:21.800Z
orchestrator: a second false-alarm source, same family: at 16:16Z the divergence check woke me with 'This clone's main has diverged from origin/main: 1 ahead, 0 behind'. By the time I looked, main was in sync: it was a fresh direct commit the automatic push hadn't reached yet. 1 ahead / 0 behind is the normal state between a commit and the next autopush tick; the check should fire only when behind > 0, or when ahead persists past an autopush tick.
