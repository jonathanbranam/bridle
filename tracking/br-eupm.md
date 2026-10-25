+++
id = "br-eupm"
title = "Orchestrator replies go to the bare role on its own home daemon, not to the sender, so the sender never reads them"
kind = "bug"
state = "built"
created_at = "2026-10-25T15:29:25.195Z"
updated_at = "2026-10-25T19:54:21.735466Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:eupm",
]
branch = "bridle/eupm"
commit = "5afe4045"
summary = """`bridle send --reply-to <id> "text"` with no recipient now routes the reply to the sender of that message, on the daemon it came from. CLI: `to` is optional with --reply-to (send_operands shifts the lone positional into the text); daemon: send_message resolves reply_route (store.forwarded_origin looks up forwarded_in for the origin machine/daemon), queues via the outbox to the sender's bare id on that daemon, and fails with the explicit `bridle send --project ...` fix on an unknown id or unreachable origin. Explicit `to` unchanged. Also inbox show's reply hint, orchestrator role text, messages.md, cli.md, CHANGELOG. Tests: two-daemon routing + unroutable failure (outbox_test.rs), operand unit tests. Caveat: --task not supported on a cross-daemon reply."""
ticket = "eupm"
+++

docs/tickets/open/orchestrator-replies-go-to-the-bare-role-on-its-own-home-dae-eupm.md

## Thread

### note · external:aide · 2026-10-25T15:29:32.220Z
Readied by aide at once (kind bug, rule ready-without-asking).

### note · agent:pm-1 · 2026-10-25T15:29:46.394Z
pm-1 brief: Model Sonnet. Blocked-by br-q7my: both change how a message to external:orchestrator is routed and checked (the send path and its docs), so they run one after another. Start by finding how --reply-to resolves its target today (bridle send, the daemon's message routing) and make a reply go to the original sender's project/daemon and identity; if the target daemon is unreachable or the sender is unknown, fail loudly with the exact fix. Add the reply instruction to workflow/base/roles/orchestrator.md. Tests: a routing test (unit, or two daemons) as in the ask. Out of scope: any change to aide waiters, the home-project refusals (q7my). Acceptance: just check.

### note · agent:eupm · 2026-10-25T19:54:19.955Z
built: 5afe4045 (branch bridle/eupm)

### note · agent:eupm · 2026-10-25T19:54:21.735Z
done: bridle send --reply-to <id> with no recipient routes to the sender's project and identity (two-daemon test + unroutable-fails test, role text, docs, CHANGELOG); just check exit 0 with BRIDLE_* unset, 1976 passed (6 skipped); afe4045
