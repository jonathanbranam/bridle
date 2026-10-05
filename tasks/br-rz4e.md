+++
id = "br-rz4e"
title = "A researcher role with WebSearch and WebFetch; the manager checks a task's tool needs before spawning (2mtr a+c)"
kind = "feature"
state = "planned"
created_at = "2026-10-05T01:02:06.369Z"
updated_at = "2026-10-05T01:02:16.756792Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Ticket: docs/tickets/open/workers-report-missing-tools-and-failed-fetches-plainly-a-re-2mtr.md (ask 2; read "What happened" and "The human's decision"). Ask 1 already landed as br-2mtr.

Approval: the human, via aide (m-5136, 2026-10-04 ~8:50 PM ET): "I like your suggestion to go with both A and C. That seems very reasonable. There's not an appropriate worker to find the tools needed, and the manager can complain to somebody, the orchestrator, I guess."

Goal:
- (a) A `researcher` role: a worker variant whose allowed tools add WebSearch and WebFetch to the worker's set. It must be spawnable in every project, including ones whose .bridle/config.toml has no [roles.researcher] (a built-in default role, as the daemon does for its other defaults; check roles-and-config.md for how defaults work), and add it to bridle's own .bridle/config.toml. Role prompt: workflow/base/roles/worker.md is fine if the role reuses it; add only what research needs (cite sources with URLs; report each failed fetch per rule report-task-failures).
- (c) workflow/base/roles/manager.md: before spawning, check what the task needs (web research -> researcher; otherwise worker). If no role has the tools a task needs, don't spawn: tell the orchestrator (`bridle send external:orchestrator --question ...`) which tool is missing.
- docs/design/agent-host/roles-and-config.md: the researcher role and its tools.
Acceptance: just check green; a test that a project with no [roles.researcher] can spawn a researcher with WebSearch and WebFetch in its allowed tools, and a plain worker still lacks them.
Model: sonnet.
Out of scope: re-running track-web's tw-sxfh (the orchestrator does it once this lands); changes to other projects' config files.
