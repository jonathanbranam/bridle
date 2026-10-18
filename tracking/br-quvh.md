+++
id = "br-quvh"
title = "Stages MVP 2: stage rules and templates, and bridle ticket next"
kind = "feature"
state = "built"
created_at = "2026-10-17T21:41:28.365Z"
updated_at = "2026-10-18T02:51:47.013349Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wquvh",
]
priority = "high"
priority_at = "2026-10-17T22:04:05.723275Z"
branch = "bridle/wquvh"
commit = "301f3a9efaec00b27bbe94c7941920d8c60d1c86"
summary = "Added 'bridle ticket next <id> [--comments] [--json]' (new module crates/bridle/src/stage_next.rs; thin next_cmd in ticket.rs, Next variant in cli.rs). Stage rules stage-proposal/design/specs/tasks in workflow/base/rules, tagged roles: [stage-writer] (a role no agent has, so they reach a writer only via next); templates are stage-<stage>.template beside them (the .template extension keeps them out of the rule loader); last layer with a template wins. Refuses on an unsigned gate, says prepared when none left. --comments = human thread comments newer than the output file's mtime (ticket file for the proposal). Docs: stages.md, cli.md, CHANGELOG. Not verified end to end against a live daemon; unit tests cover pick, layout and template layering."
ticket = "quvh"
+++

docs/tickets/open/stages-mvp-2-stage-rules-and-templates-and-bridle-ticket-nex-quvh.md

## Thread

### note · external:advisor/tj7q · 2026-10-17T22:03:56.107Z
The human, 2026-10-17 ~5:15 PM ET, to advisor (tj7q): "Yes approved. Let's go" (shared understanding of epic tj7q, and ready steps 1-5 so the build can start today; the human wants the MVP done today to use tomorrow). Readied by human via advisor (tj7q).

### note · external:advisor/tj7q · 2026-10-17T22:04:05.723Z
priority: normal -> high

### note · agent:wquvh · 2026-10-18T02:51:46.583Z
built: 301f3a9efaec00b27bbe94c7941920d8c60d1c86 (branch bridle/wquvh)

### note · agent:wquvh · 2026-10-18T02:51:47.013Z
done: ticket next + 4 stage rules/templates; just check exit 0, 1845 tests passed, on merged tip 301f3a9efaec00b27bbe94c7941920d8c60d1c86
