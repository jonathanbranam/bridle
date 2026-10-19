+++
id = "br-bdwh"
title = "No command to change which stages of a version 2 ticket need the human's review"
kind = "feature"
state = "integrated"
created_at = "2026-10-18T21:24:07.523Z"
updated_at = "2026-10-19T23:26:46.554266Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wbdwh",
]
branch = "bridle/wbdwh"
commit = "2e9761fe5e3173152c6f63868e8b9483758ebc3b"
summary = "Added 'bridle ticket set <id> review <list>' (set_review in crates/bridle/src/ticket.rs): the list replaces the stages marked review: human, empty clears; refuses unlisted stages (naming valid ones), a change to an already-signed stage, and v1 tickets. No wire change. Docs: cli.md, stages.md; CHANGELOG Added. Test: set_review_adds_removes_and_refuses."
ticket = "bdwh"
+++

docs/tickets/open/no-command-to-change-which-stages-of-a-version-2-ticket-need-bdwh.md

## Thread

### note · agent:pm-1 · 2026-10-18T21:24:49.665Z
Brief (pm-1). Model: Sonnet. Human approved the ask ("Yes approve", per the ticket). Build `bridle ticket set <id> review <comma list>` reusing `set` and the `ticket new --review` list parser; it sets the stages needing the human's signature on an existing version 2 ticket. Decisions (so the worker needn't ask): the list REPLACES the review set (empty value clears it); a stage the ticket doesn't list is refused with the list of valid stages; changing review on an already-signed stage is refused with a clear message (signed stages stay as signed). Files: the ticket set code in crates/bridle and crates/bridle-daemon (grep "blocked_by" in the set handler, and `--review` in ticket new), docs/design/stages.md, docs/design/cli.md, CHANGELOG. If it changes the wire type, update bridle-api types.rs, clients and daemon together. Tests: add, remove, unknown stage refused, signed stage refused, v1 ticket refused. Acceptance: just check. Out of scope: changing which stages exist.
Touches stage code like br-fyww and br-upxq: run after them.

### note · agent:wbdwh · 2026-10-19T23:21:22.894Z
built: 85eeb293 (branch bridle/wbdwh)

### note · agent:wbdwh · 2026-10-19T23:21:22.984Z
done: ticket set review; just check exit 0, 1901 tests passed, tip 85eeb293

### note · agent:manager-2 · 2026-10-19T23:26:32.300Z
integrated: 2e9761fe5e3173152c6f63868e8b9483758ebc3b (branch bridle/wbdwh)

### note · agent:manager-2 · 2026-10-19T23:26:46.554Z
cleanup: removed agent wbdwh, branch bridle/wbdwh
