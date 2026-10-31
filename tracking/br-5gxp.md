+++
id = "br-5gxp"
title = "send --reply-to with no recipient fails 'to is required' for a message from a visiting orchestrator"
kind = "bug"
state = "built"
created_at = "2026-10-31T03:28:03.481Z"
updated_at = "2026-10-31T04:23:09.878786Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w5gxp",
]
branch = "bridle/w5gxp"
commit = "89837461"
summary = """Not reproducible on current code for a local sender through the real CLI (new e2e test). The reported error text, exactly "`to` is required", predates br-eupm (current daemons add "(or `reply_to`...)"), so the answering daemon was an older build than b4d46c05, not a dropped hop. Fix: when a bare --reply-to gets that exact error, the CLI fails naming the daemon's version, its own, "restart it (bridle daemon restart)", and the working `bridle send <sender> --reply-to <id> <text>`. Versions are crate semver, so two builds of one release look alike; the exact error text is the real signal. Tests: cli_e2e local-sender bare reply; cli_e2e fake old daemon. Visitor and forwarded-sender routes need two daemons and are covered at the daemon in outbox_test.rs (br-eupm, br-3sta), not through the CLI. Docs: messages.md; CHANGELOG Fixed."""
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

### note · agent:w5gxp · 2026-10-31T04:18:06.804Z
built: 89837461 (branch bridle/w5gxp)

### note · agent:w5gxp · 2026-10-31T04:18:06.846Z
done: stale-daemon cause confirmed by error text, CLI now names both versions and the working command; check exit 0, 1981 tests, 89837461

### note · agent:manager-2 · 2026-10-31T04:18:12.983Z
Main moved (br-5hca landed, 7085fa99). Merge main into your branch, run just check with BRIDLE_* unset, commit, and message me the new sha. Also in the CHANGELOG: your line sits in the lower Fixed section of Unreleased; fine, keep it.

### note · agent:w5gxp · 2026-10-31T04:23:09.878Z
main merged (7085fa99); check exit 0, 1982 tests; new sha 9e470d89
