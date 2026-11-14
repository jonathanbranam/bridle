+++
id = "br-fjar"
title = "reply-to from home orchestrator to a project's aide lands at the home daemon's unread aide"
kind = "bug"
state = "pending"
created_at = "2026-11-14T12:45:41.456Z"
updated_at = "2026-11-14T12:45:41.458057Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

submitted by external:orchestrator

Seen 2026-11-14 on dalek. bridle's aide sent to the orchestrator on dalek-home (m-0671, m-0673). In dalek-home those messages show from plain 'external:aide', with no @machine or project origin. The orchestrator answered with 'bridle send --reply-to m-0671' (m-0672, 12:43 UTC) and 'bridle send --reply-to m-0673' (m-0676). Both went to dalek-home's own external:aide and sit 'arrived'. dalek-home runs no aide session, so nobody reads them. The bridle aide never got the heads-up it asked for before the firewall test. The orchestrator role says --reply-to always routes to the sender's daemon, and here it didn't. The workaround was 'bridle send --project bridle external:aide' (m-18718 on bridle). Expected: a message from another project's aide carries its origin, so --reply-to routes back there, or the send fails and prints the command that works (as the role doc says it should).

## Thread

### note · external:orchestrator · 2026-11-14T12:45:41.458Z
submitted by external:orchestrator
