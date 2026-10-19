+++
id = "br-ggbx"
title = "Relay the human's comments through an agent: approve with a comment, signed by the human via the agent"
kind = "fix"
state = "built"
created_at = "2026-10-19T11:08:33.690Z"
updated_at = "2026-10-19T21:17:12.879714Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wggbx",
]
branch = "bridle/wggbx"
commit = "602ba6b443d6411125f18a18584dd0e41950174f"
summary = "ThreadEntry gains a for_human mark (types.rs, stored as a 4th 'for-human' part of the thread header); NoteTicketRequest.for_human; the daemon (server.rs may_relay_for_human) records it only for the human, external:advisor*, external:aide, external:orchestrator and product-manager agents, else 403. CLI: ticket approve --comment/--comment-file (comment note posted first so a refusal changes nothing), ticket comment --for-human, ticket changes text marked when an agent runs it; next --comments counts for_human or human entries. Docs: stages.md, cli.md, CHANGELOG. Caveat: a worker running 'ticket changes <text>' is now refused."
ticket = "ggbx"
+++

docs/tickets/open/relay-the-human-s-comments-through-an-agent-approve-with-a-c-ggbx.md

## Thread

### note · agent:pm-1 · 2026-10-19T11:09:16.473Z
Brief (pm-1). Model: Sonnet. The human asked for this in the Stages try ("signed by me via you"), so rank it high, ahead of br-bdwh. Build the ask as written: `ticket approve --comment/--comment-file`, `ticket comment --for-human`, `ticket changes` recorded the same way, all stored as "human via <principal>"; `next_cmd --comments` (crates/bridle/src/ticket.rs) counts them. Decisions so the worker needn't ask: the stored author field stays the real principal and a new marker records "for human" (do not forge `from`); `--comments` matches on that marker as well as the existing human match; allowed principals are external:advisor*, external:aide, external:orchestrator and the product-manager role, refused for workers and designers with a clear error. If this changes a wire type, update bridle-api types.rs, clients and daemon together. Docs: stages.md, cli.md, CHANGELOG. Tests: approve with comment records both, comment --for-human appears in next --comments, a worker is refused. Acceptance: just check. Touches the same ticket code as br-bdwh: run before it.

### note · agent:wggbx · 2026-10-19T21:17:12.796Z
built: 602ba6b443d6411125f18a18584dd0e41950174f (branch bridle/wggbx)

### note · agent:wggbx · 2026-10-19T21:17:12.879Z
done: relay the human's comments (approve --comment, comment --for-human, changes); just check exit 0, 1898 tests passed; 602ba6b443d6411125f18a18584dd0e41950174f
