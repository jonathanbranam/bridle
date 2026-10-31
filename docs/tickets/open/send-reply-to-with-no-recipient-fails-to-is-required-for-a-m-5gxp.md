---
id: br-5gxp
title: send --reply-to with no recipient fails 'to is required' for a message from a visiting orchestrator
kind: bug
created: 2026-10-31
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-jpph, br-eupm]
---

## The ask

Third report of the same symptom (br-jpph was closed "not reproducible, old daemon", then reproduced again on 2026-10-30 on a same-daemon message). Seen by orchestrator@dalek on dalek-home, 2026-10-30 ~11:30 PM ET, daemon at b4d46c05 (after br-eupm, br-jpph, br-3sta all merged).

m-0513 arrived on dalek-home from `external:orchestrator@remus` (a visitor, forwarded via remus's home daemon). `bridle send --reply-to m-0513 "..."` (no recipient) failed with `Error: bad_request: `to` is required`. Workaround: `bridle send external:orchestrator@remus --reply-to m-0513 "..."`. Earlier: the same on a local message from external:aide (m-0479).

Key fact for the worker: the error text `to is required` comes from the daemon, so the request arrived with no `to` and the daemon's reply_route did not fill it in (or the CLI sent no `reply_to`). Do not conclude "old daemon" again: reproduce against the daemon as built from main with the CLI binary the orchestrator actually runs (check `which bridle`, its version vs the daemon's, and that the self-upgrade replaces the CLI binary the orchestrator's shell uses; a CLI older than the daemon would still hit this). Then cover: a local sender, a visitor `<name>@<machine>`, and a forwarded sender. Find which hop drops it: CLI send_operands (does it shift the lone positional into the text and set reply_to?), the daemon's send_message/reply_route, forwarded_origin lookup.

Fix: bare `--reply-to` fills in the referenced message's sender (the visitor `<name>@<machine>` or the local principal); when it truly cannot, fail with the exact working command, as the role docs say. If the cause is a stale CLI binary, the fix is making the version mismatch visible (a clear error naming both versions) and the upgrade replace what agents run.

Acceptance: tests for the three sender kinds above through the real CLI path (not only the daemon API); `just check`. Model: Sonnet. Files: crates/bridle (send CLI), crates/bridle-daemon (send_message, reply_route), outbox_test.rs, docs/design/agent-host/messages.md. Out of scope: --task on cross-daemon replies.
