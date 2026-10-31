+++
id = "br-5gxp"
title = "send --reply-to with no recipient fails 'to is required' for a message from a visiting orchestrator"
kind = "bug"
state = "claimed"
created_at = "2026-10-31T03:28:03.481Z"
updated_at = "2026-10-31T03:42:54.279859Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w5gxp",
]
ticket = "5gxp"
+++

docs/tickets/open/send-reply-to-with-no-recipient-fails-to-is-required-for-a-m-5gxp.md

submitted by external:orchestrator

Seen by orchestrator@dalek on dalek-home, 2026-10-31 ~03:30 UTC, daemon at b4d46c05 (after br-3sta, forwarded-reply routing, merged).

## What happened

m-0513 arrived on dalek-home from `external:orchestrator@remus` (a visitor, forwarded via remus's home daemon). Replying as the orchestrator role tells you to:

    bridle send --reply-to m-0513 "..."

failed with:

    Error: bad_request: `to` is required

`bridle send --help` says that with no recipient, `--reply-to` routes to the sender on the project it came from. The orchestrator role also says that if the reply can't be routed, the send fails *and prints the command that works*; neither happened.

Workaround that worked: `bridle send external:orchestrator@remus --reply-to m-0513 "..."` (outbox o-0017).

## The ask

Make `--reply-to` with no recipient fill in the sender of the referenced message (here a `<name>@<machine>` visitor), or, when it can't, fail with the exact command to use, as documented.

## Verify

A test: a message from a visiting principal, replied to with `--reply-to` and no recipient, is sent to that visitor.

## Thread

### note · external:orchestrator · 2026-10-31T03:28:03.482Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-31T03:28:38.298Z
Readied by orchestrator (bug; rule ready-without-asking).
