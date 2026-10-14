+++
id = "br-drvx"
title = "Statusline reports from sessions outside any bridle project are counted in a machine-level bucket (ktgd decision 2)"
kind = "feature"
state = "claimed"
created_at = "2026-10-13T23:05:27.092Z"
updated_at = "2026-10-14T05:44:16.381919Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:drvx",
]
ticket = "drvx"
+++

docs/tickets/open/statusline-reports-from-sessions-outside-any-bridle-project-drvx.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:35.293Z
Brief (pm-1). Model: Sonnet. Size: S-M. Blocked by br-mksa only because both edit the statusline report path; start after it merges (I add the edge). The ask and ktgd decision 2 are the spec. Start: `bridle statusline` in crates/bridle/src (how it finds the project's daemon and drops the rest) and the discovery code in bridle-api. Simplest design: when no project daemon is found for the session's directory, post to one per-machine daemon chosen by a rule you document (e.g. a `[machine]` setting naming a project, defaulting to the first project in the registry), recorded with project = "outside any project". Never block or slow the status line (short timeout, fail silent as today). Test: a session in a non-bridle directory is recorded and shows under the bucket in `bridle usage`. Docs: usage-and-budget.md, cli.md, CHANGELOG.md. If this adds a config key, say how existing machines get a sane default without edits. Out of scope: desktop app and claude.ai sessions.
