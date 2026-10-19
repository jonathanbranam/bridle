+++
id = "br-bdwh"
title = "No command to change which stages of a version 2 ticket need the human's review"
kind = "feature"
state = "claimed"
created_at = "2026-10-18T21:24:07.523Z"
updated_at = "2026-10-19T23:02:06.014169Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wbdwh",
]
ticket = "bdwh"
+++

docs/tickets/open/no-command-to-change-which-stages-of-a-version-2-ticket-need-bdwh.md

## Thread

### note · agent:pm-1 · 2026-10-18T21:24:49.665Z
Brief (pm-1). Model: Sonnet. Human approved the ask ("Yes approve", per the ticket). Build `bridle ticket set <id> review <comma list>` reusing `set` and the `ticket new --review` list parser; it sets the stages needing the human's signature on an existing version 2 ticket. Decisions (so the worker needn't ask): the list REPLACES the review set (empty value clears it); a stage the ticket doesn't list is refused with the list of valid stages; changing review on an already-signed stage is refused with a clear message (signed stages stay as signed). Files: the ticket set code in crates/bridle and crates/bridle-daemon (grep "blocked_by" in the set handler, and `--review` in ticket new), docs/design/stages.md, docs/design/cli.md, CHANGELOG. If it changes the wire type, update bridle-api types.rs, clients and daemon together. Tests: add, remove, unknown stage refused, signed stage refused, v1 ticket refused. Acceptance: just check. Out of scope: changing which stages exist.
Touches stage code like br-fyww and br-upxq: run after them.
