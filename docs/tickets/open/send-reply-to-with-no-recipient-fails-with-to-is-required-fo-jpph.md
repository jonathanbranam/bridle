---
id: br-jpph
title: send --reply-to with no recipient fails with 'to is required' for a message from another machine's orchestrator
kind: bug
created: 2026-10-29
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Found by orchestrator@dalek, 2026-10-28 ~9:30 PM ET. Follows br-eupm (bare `--reply-to` routing), which landed in 09cd736b.

On dalek-home, `bridle send --reply-to m-0464 "..."` (no recipient) failed with `error: bad_request: `to` is required`. m-0464 came from `external:orchestrator@vesta` (forwarded, reply_to null). Per the orchestrator role, a bare --reply-to routes to the sender on its daemon, or fails and prints the working command. Neither happened. Workaround: `bridle send --project vesta-home external:orchestrator --reply-to m-0464 "..."`.

First check whether the installed `bridle` binary on dalek was older than 09cd736b (the CLI is what shifts the lone positional into the text); if so, reproduce on current main before changing code. If it still fails, find why the forwarded-origin lookup (store.forwarded_origin / reply_route) misses a cross-machine sender whose forwarded_in row is present or absent, and fix it, or fail with the working `bridle send --project ...` command as documented.

Acceptance: a test where a bare `--reply-to` answers a message forwarded from another machine's orchestrator (delivered, or the error prints the working command); `just check`. Model: Sonnet. Files: the send path from br-eupm (crates/bridle CLI send_operands, daemon send_message/reply_route, outbox_test.rs). Out of scope: --task on cross-daemon replies.
