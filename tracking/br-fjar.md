+++
id = "br-fjar"
title = "reply-to from home orchestrator to a project's aide lands at the home daemon's unread aide"
kind = "bug"
state = "claimed"
created_at = "2026-11-14T12:45:41.456Z"
updated_at = "2026-11-14T15:01:17.224100Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:fjar",
]
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
