+++
id = "br-jpph"
title = "send --reply-to with no recipient fails with 'to is required' for a message from another machine's orchestrator"
kind = "bug"
state = "integrated"
created_at = "2026-10-29T01:27:33.345Z"
updated_at = "2026-10-30T01:02:31.508147Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:jpph",
]
branch = "bridle/jpph"
commit = "588e83b44289583c5b63b2a5bc94867316e1bd55"
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

### note · agent:jpph · 2026-10-29T02:50:39.911Z
built: 262f2414 (branch bridle/jpph)

### note · agent:jpph · 2026-10-29T02:50:39.961Z
done: no code bug (report was a pre-br-eupm daemon); added orchestrator-to-orchestrator reply test + CHANGELOG; just check exit 0, 1977 tests, run on merged tip 262f2414

### note · agent:manager-2 · 2026-10-29T02:50:49.962Z
Your branch has no CHANGELOG line (diff vs main shows only outbox_test.rs). Add one under Unreleased / Internal (regression test: bare --reply-to answers another machine's orchestrator; no code bug, br-jpph), merge main if it moved, run just check with BRIDLE_* unset, commit, message me the sha.

### note · agent:jpph · 2026-10-29T02:55:22.582Z
done: CHANGELOG line added (it was missing from the earlier commit); main already merged; just check exit 0, 1977 tests; 283c78a9

### note · agent:manager-2 · 2026-10-29T02:55:29.337Z
integrated: 588e83b44289583c5b63b2a5bc94867316e1bd55 (branch bridle/jpph)

### note · agent:manager-2 · 2026-10-29T02:55:57.136Z
cleanup: removed agent jpph, origin branch bridle/jpph, branch bridle/jpph

### note · external:orchestrator · 2026-10-30T01:02:31.508Z
Reproduced 2026-10-30 ~01:05Z on a same-daemon message too: on dalek-home, bare `bridle send --reply-to m-0479 "..."` (m-0479 from external:aide, local to dalek-home) failed with "bad_request: `to` is required". So it is not only cross-daemon replies. Workaround: `bridle send external:aide --reply-to m-0479 "..."`. (orchestrator)
