+++
id = "br-yfjc"
title = "Every agent talking to the human links each ticket and task it names to the bridle UI, from a configured base URL"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T01:19:32.477Z"
updated_at = "2026-10-05T03:54:11.084518Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
branch = "bridle/link-ids"
commit = "8c7cc7689f3682225d7e016d679978331e30dcf2"
summary = "Added 'bridle link <id>' (crates/bridle/src/link.rs): task IDs (contain '-') -> <base>/task?id=<id>, ticket IDs -> <base>/ticket?project=<p>&id=<id> (project via --project/env/cwd workspace). Base URL is [gateway] public_url, project .bridle/config.toml over ~/.bridle/config.toml (config::ui_base_url); unset prints nothing, exit 0. GatewaySection now accepts public_url (deny_unknown_fields). New rule link-ids-for-the-human, one-line pointers in aide/advisor/orchestrator prompts; docs cli.md, roles-and-config.md, CHANGELOG. Tests: tests/link_test.rs (ticket, task, unset). Note: no manager role prompt line was added (task named aide, advisor, orchestrator only)."
+++

original id: yfjc
docs/tickets/open/every-agent-talking-to-the-human-links-each-ticket-and-task-yfjc.md

Approval: the human, via bridle-ui's aide (m-5179, 2026-10-04 ~9:20 PM ET): "please write a ticket and ask to get that scheduled so that I have links every time an agent that's talking to me refers to a ticket or a task" and "The address to open, like the URL, should be project configuration or bridle configuration".

Goal:
- Config: a UI base URL in ~/.bridle/config.toml (e.g. `[gateway] public_url = "http://dalek.tailbc91f5.ts.net:7878"`), optional per-project override in .bridle/config.toml. Unset: no links, nothing breaks.
- `bridle link <id>` prints the URL for a ticket ID (`/ticket?project=<p>&id=<id>`, bridle-ui ui-sau7) or a task ID (`/task?id=<id>`, bridle-ui ui-umaq); prints nothing and exits 0 when no base URL is set. Agents call it instead of building URLs by hand. (The UI routes land separately; the links work once they do.)
- A base rule (workflow/base/rules/, e.g. link-ids-for-the-human.md) for roles that talk to the human (aide, advisor, orchestrator; managers in messages to the human): when naming a ticket or task to the human, add its link from `bridle link`. Point the aide, advisor and orchestrator role prompts at it in one line each.
- docs/design/cli.md and roles-and-config.md for the key and command.
Acceptance: just check green; tests: `bridle link` for a ticket ID, a task ID, and with no base URL.
Model: sonnet (small).
Out of scope: the UI routes themselves (bridle-ui ui-sau7, ui-umaq); setting the URL on dalek (the human does it: a to-do after landing).

## Thread

### note · external:orchestrator · 2026-10-05T02:27:09.787Z
priority: normal -> high

### note · external:orchestrator · 2026-10-05T02:27:09.807Z
Raised to high by orchestrator: the human, via aide (m-5240, 2026-10-04 ~10:15 PM ET): "I want that done as soon as possible. If you have the ticket, you can just read it and start following instructions right now." Start it next.

### note · agent:link-ids · 2026-10-05T03:23:06.257Z
done: bridle link + [gateway] public_url + rule link-ids-for-the-human; just check exit 0, 1194 tests passed; 506ef362

### note · agent:manager-2 · 2026-10-05T03:54:11.084Z
integrated: 8c7cc7689f3682225d7e016d679978331e30dcf2 (branch bridle/link-ids)
