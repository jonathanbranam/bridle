+++
id = "br-jpph"
title = "send --reply-to with no recipient fails with 'to is required' for a message from another machine's orchestrator"
kind = "bug"
state = "claimed"
created_at = "2026-10-29T01:27:33.345Z"
updated_at = "2026-10-29T02:46:33.510620Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:jpph",
]
summary = "Not a code bug: the reported 'to is required' text is the pre-br-eupm daemon's; current main routes a bare --reply-to correctly. Added outbox_test a_reply_to_another_machines_orchestrator_goes_back_to_it (orchestrator on A -> orchestrator on B, B replies bare, lands in A's orchestrator inbox) and a CHANGELOG Internal line. No docs needed. Fix for the user: upgrade the daemon on the machine that failed."
ticket = "jpph"
+++

docs/tickets/open/send-reply-to-with-no-recipient-fails-with-to-is-required-fo-jpph.md

submitted by external:orchestrator

Found by orchestrator@dalek, 2026-10-29 01:3x UTC.

## The ask

On dalek-home, `bridle send --reply-to m-0464 "..."` (no recipient) failed with `error: bad_request: \`to\` is required`. m-0464 came from `external:orchestrator@vesta` (forwarded, reply_to null). The orchestrator role says a bare --reply-to routes to the sender's principal on its daemon, and if it can't route, the send fails and prints the command that works. Neither happened. Workaround that worked: `bridle send --project vesta-home external:orchestrator --reply-to m-0464 "..."`.

Fix: route the reply to the cross-machine sender, or fail with the working command as documented.

Verify: a bare --reply-to to a message from orchestrator@<other machine> is delivered (or prints the working command).

## Thread

### note · external:orchestrator · 2026-10-29T01:27:33.346Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-29T01:28:09.438Z
Readied by orchestrator (dalek): bug, one clear fix; rule ready-without-asking.

### note · agent:jpph · 2026-10-29T02:17:54.544Z
Not reproducible on current main: the reported error text is the pre-br-eupm daemon's. Added test a_reply_to_another_machines_orchestrator_goes_back_to_it (passes), CHANGELOG line. Full just check running (earlier run hit 30m limit queued on the nextest lock); will commit, summarise and report when green.
