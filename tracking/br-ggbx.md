+++
id = "br-ggbx"
title = "Relay the human's comments through an agent: approve with a comment, signed by the human via the agent"
kind = "fix"
state = "open"
created_at = "2026-10-19T11:08:33.690Z"
updated_at = "2026-10-19T11:09:16.473832Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
ticket = "ggbx"
+++

docs/tickets/open/relay-the-human-s-comments-through-an-agent-approve-with-a-c-ggbx.md

## Thread

### note · agent:pm-1 · 2026-10-19T11:09:16.473Z
Brief (pm-1). Model: Sonnet. The human asked for this in the Stages try ("signed by me via you"), so rank it high, ahead of br-bdwh. Build the ask as written: `ticket approve --comment/--comment-file`, `ticket comment --for-human`, `ticket changes` recorded the same way, all stored as "human via <principal>"; `next_cmd --comments` (crates/bridle/src/ticket.rs) counts them. Decisions so the worker needn't ask: the stored author field stays the real principal and a new marker records "for human" (do not forge `from`); `--comments` matches on that marker as well as the existing human match; allowed principals are external:advisor*, external:aide, external:orchestrator and the product-manager role, refused for workers and designers with a clear error. If this changes a wire type, update bridle-api types.rs, clients and daemon together. Docs: stages.md, cli.md, CHANGELOG. Tests: approve with comment records both, comment --for-human appears in next --comments, a worker is refused. Acceptance: just check. Touches the same ticket code as br-bdwh: run before it.
