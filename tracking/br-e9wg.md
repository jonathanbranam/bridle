+++
id = "br-e9wg"
title = "Automatic push alarms on a one-off race, and its message hides git's reason"
kind = "bug"
state = "integrated"
created_at = "2026-10-11T14:29:42.746Z"
updated_at = "2026-10-17T11:37:14.222543Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:we9wg",
]
commit = "356bc398"
summary = "autopush.rs: first_line picks git's '! ' or 'error:' reason line (fallback first line), the reason is logged, and tick() alarms only when the push fails on a second tick (once per repeated error). divergence.rs: ahead-only is reported only if it persists to the next look; behind > 0 reports at once. Tests added in both; docs in operating-model.md and CHANGELOG updated. No migration. just check: 1529 tests passed."
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

### note · agent:we9wg · 2026-10-11T17:49:03.474Z
done: autopush alarms on 2nd consecutive failure and names git's reason; divergence ahead-only waits a look; just check exit 0, 1529 tests passed, main merged; 8832901a

### note · agent:manager-2 · 2026-10-11T18:14:59.717Z
Landing was interrupted by a daemon restart and main moved (docs only). Merge main, run just check, message me the sha right away.

### note · agent:we9wg · 2026-10-11T18:18:53.959Z
main merged; just check exit 0, 1529 passed; sha 8bc2b5a1

### note · agent:manager-2 · 2026-10-11T21:16:16.608Z
integrated: 356bc398

### note · external:advisor/product-manager · 2026-10-17T11:37:14.222Z
Verified by advisor (product-manager) 2026-10-17 ~8 AM ET: task integrated, landed on origin/main, every item of the ask done (or confirmed done by the human). Resolving.
