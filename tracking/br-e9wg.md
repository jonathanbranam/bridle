+++
id = "br-e9wg"
title = "Automatic push alarms on a one-off race, and its message hides git's reason"
kind = "bug"
state = "planned"
created_at = "2026-10-11T14:29:42.746Z"
updated_at = "2026-10-11T17:19:06.531379Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "e9wg"
+++

Ticket: docs/tickets/open/automatic-push-alarms-on-a-one-off-race-and-its-message-hide-e9wg.md (read it; human approved 2026-10-11) plus the second false-alarm source on the task thread (the divergence check). Low priority, Haiku, small.

Build in crates/bridle-daemon/src/autopush.rs: (1) the failure message names git's reason line ('! [rejected] ...' or 'error: ...'), falling back to the first line; log the reason in the daemon log too. (2) tick() alarms only when the push still fails on the NEXT tick (the branch is still ahead of origin), so a one-off race between pushers stays quiet. (3) The divergence check ("main has diverged: N ahead, 0 behind") fires only when behind > 0, or when ahead persists past an autopush tick; find it (grep diverged in crates/bridle-daemon).
Files: autopush.rs and the divergence check's file, tests there, docs/design/agent-host/ where autopush is described, CHANGELOG. Migration: none.
Acceptance: just check passes; tests: first push fails then next succeeds sends no alarm; two failures in a row alarm once with git's reason; 1 ahead 0 behind does not alarm on first sight, behind > 0 does. Out of scope: other alarms.

## Thread

### note · external:orchestrator · 2026-10-11T16:16:21.800Z
orchestrator: a second false-alarm source, same family: at 16:16Z the divergence check woke me with 'This clone's main has diverged from origin/main: 1 ahead, 0 behind'. By the time I looked, main was in sync: it was a fresh direct commit the automatic push hadn't reached yet. 1 ahead / 0 behind is the normal state between a commit and the next autopush tick; the check should fire only when behind > 0, or when ahead persists past an autopush tick.

### note · external:advisor/product-manager · 2026-10-11T17:18:05.391Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve as recommended". Readied, low.
