+++
id = "br-eupm"
title = "Orchestrator replies go to the bare role on its own home daemon, not to the sender, so the sender never reads them"
kind = "bug"
state = "planned"
created_at = "2026-10-25T15:29:25.195Z"
updated_at = "2026-10-25T15:43:06.019795Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "eupm"
+++

docs/tickets/open/orchestrator-replies-go-to-the-bare-role-on-its-own-home-dae-eupm.md

## Thread

### note · external:aide · 2026-10-25T15:29:32.220Z
Readied by aide at once (kind bug, rule ready-without-asking).

### note · agent:pm-1 · 2026-10-25T15:29:46.394Z
pm-1 brief: Model Sonnet. Blocked-by br-q7my: both change how a message to external:orchestrator is routed and checked (the send path and its docs), so they run one after another. Start by finding how --reply-to resolves its target today (bridle send, the daemon's message routing) and make a reply go to the original sender's project/daemon and identity; if the target daemon is unreachable or the sender is unknown, fail loudly with the exact fix. Add the reply instruction to workflow/base/roles/orchestrator.md. Tests: a routing test (unit, or two daemons) as in the ask. Out of scope: any change to aide waiters, the home-project refusals (q7my). Acceptance: just check.
