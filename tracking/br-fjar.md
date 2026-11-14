+++
id = "br-fjar"
title = "reply-to from home orchestrator to a project's aide lands at the home daemon's unread aide"
kind = "bug"
state = "integrated"
created_at = "2026-11-14T12:45:41.456Z"
updated_at = "2026-11-14T15:22:45.199267Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:fjar",
]
branch = "bridle/fjar"
commit = "ca19ae31e1a04083c96b7e69e1023ecaffba6cd3"
summary = "Chose option 2 (simpler, no wire change). reply_route in server.rs now refuses a --reply-to to an external:aide/advisor message with no forwarded origin when no such session runs on this daemon, and prints 'bridle send --project <project> external:aide --reply-to <id>'. Cause: a cross-project 'send --project' goes straight to the target daemon as the bare role, so it has no origin. Caveat: a same-project reply to an aide whose session has exited now also errors instead of queueing. Test in outbox_test.rs; cli.md updated; role doc claim already matches."
ticket = "fjar"
+++

docs/tickets/open/reply-to-from-home-orchestrator-to-a-project-s-aide-lands-at-fjar.md

submitted by external:orchestrator

Seen 2026-11-14 on dalek. bridle's aide sent to the orchestrator on dalek-home (m-0671, m-0673). In dalek-home those messages show from plain 'external:aide', with no @machine or project origin. The orchestrator answered with 'bridle send --reply-to m-0671' (m-0672, 12:43 UTC) and 'bridle send --reply-to m-0673' (m-0676). Both went to dalek-home's own external:aide and sit 'arrived'. dalek-home runs no aide session, so nobody reads them. The bridle aide never got the heads-up it asked for before the firewall test. The orchestrator role says --reply-to always routes to the sender's daemon, and here it didn't. The workaround was 'bridle send --project bridle external:aide' (m-18718 on bridle). Expected: a message from another project's aide carries its origin, so --reply-to routes back there, or the send fails and prints the command that works (as the role doc says it should).

## Thread

### note · external:orchestrator · 2026-11-14T12:45:41.458Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-11-14T12:46:49.252Z
The human, 2026-11-14, via aide (m-0677), verbatim: "that's interesting on the addressing side; the orchestrator has no aide; I think I want to keep it that way". So the fix routes a reply to external:aide back to the sending project's aide, or refuses or flags a reply to an aide that doesn't exist on dalek-home. Don't add an aide to dalek-home.

### note · external:orchestrator · 2026-11-14T12:51:15.586Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-14T12:51:20.236Z
Readied by orchestrator (bug, clear ask; rule ready-without-asking). The file was already committed (2ec3b494).

### note · agent:fjar · 2026-11-14T15:22:21.500Z
built: 13b3b265 (branch bridle/fjar)

### note · agent:fjar · 2026-11-14T15:22:21.550Z
done: option 2, reply to originless sessionless aide/advisor errors with the working command; 13b3b265; just check (env -i) exit 0, 2010 tests passed

### note · agent:manager-2 · 2026-11-14T15:22:29.990Z
integrated: ca19ae31e1a04083c96b7e69e1023ecaffba6cd3 (branch bridle/fjar)

### note · agent:manager-2 · 2026-11-14T15:22:45.199Z
cleanup: removed agent fjar, origin branch bridle/fjar, branch bridle/fjar
