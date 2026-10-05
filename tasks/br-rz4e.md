+++
id = "br-rz4e"
title = "A researcher role with WebSearch and WebFetch; the manager checks a task's tool needs before spawning (2mtr a+c)"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T01:02:06.369Z"
updated_at = "2026-10-05T17:37:58.855625Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/researcher-role"
commit = "0278584a3b00353255323be1fc36cf6a3090869d"
summary = "Added built-in researcher role (Role::researcher_default in config.rs: worker defaults + WebSearch/WebFetch in allowed_tools and tools; reuses worker.md prompt via default_role_prompts fallback; preamble sentence on URLs and failed fetches). Added [roles.researcher] to bridle's .bridle/config.toml, manager.md role-picking/missing-tool step, roles-and-config.md, CHANGELOG. Test: researcher_is_a_built_in_worker_with_web_tools. No other docs needed changes."
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

## Thread

### note · agent:researcher-role · 2026-10-05T17:32:52.808Z
done: built-in researcher role + manager.md tool check + docs + CHANGELOG; just check exit 0, 1245 tests run (1245 passed), commit 1159e54a

### note · agent:manager-2 · 2026-10-05T17:32:58.060Z
integrated: 0278584a3b00353255323be1fc36cf6a3090869d (branch bridle/researcher-role)

### note · agent:manager-2 · 2026-10-05T17:37:58.855Z
cleanup: removed agent researcher-role, branch bridle/researcher-role
