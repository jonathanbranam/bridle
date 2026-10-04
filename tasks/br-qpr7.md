+++
id = "br-qpr7"
title = "bridle uses its own specs, starting with how every command finds its project"
kind = "feature"
state = "integrated"
created_at = "2026-10-04T18:39:04.274Z"
updated_at = "2026-10-04T20:50:24.966148Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
branch = "bridle/specs-first"
commit = "2d9adce22f660ebf0add1d610fef22685ee729e7"
summary = "Bridle adopts its own specs. New design/specs/project-resolution.md (3 requirements, 6 executable scenarios): every project-scoped command picks its project from --project, then $BRIDLE_PROJECT, then the cwd's workspace; refuses (naming --project) otherwise, never a hard-coded bridle; a new command must be classified. crates/bridle/tests/project_resolution_test.rs reads the whole command tree (subcommands included) from --help and runs every daemon-reaching and session-launching leaf from a temp workspace against fake daemons (connection counts show which project it reached; a stub claude shows the session's project), with the flag, the env var, and outside a workspace; naming commands (launchd, systemd on Linux, ticket new) are checked for x inside and the folder name outside. CLASSES in that test is the table a new command must join (parallels SCOPES in project.rs). `just check` now has a specs-check step (spec check --require-ids, spec coverage --tests crates --require-all). Docs: cli.md, specs/spec-flow/specs-to-tests status lines, CHANGELOG. Findings, not fixed: `migrate` and `review add|list|remove|resolve` act on the cwd's repo, not a resolved project (listed as Skip, though SCOPES calls them Daemon); `orchestrator prime` succeeds outside a workspace (prints the role, reaches no daemon); `advisor start` outside a workspace prints a command with no --project instead of refusing. Open question for the thread: should the spec flow (tasks edit design/specs/ in place) become a rule for bridle's own tasks, and which capability is spec'd next."
+++

original id: qpr7
Build docs/tickets/open/bridle-uses-its-own-specs-starting-with-how-every-command-fi-qpr7.md (read it, plus docs/design/specs.md, specs-to-tests.md, spec-flow.md 'Adopting, in order'). Bridle adopts its own specs: create design/specs/ for bridle, with a first spec for how every project-scoped command finds its project (--project, then BRIDLE_PROJECT, then the cwd's workspace, refuse otherwise, never a hard-coded bridle). Add an EXECUTABLE scenario that runs every project-scoped command, subcommands included, from a temp workspace for project X with no --project and checks it resolved X (or refused outside any workspace); link spec to tests so bridle spec check and bridle spec coverage pass. Builds on crates/bridle/src/project.rs and the SCOPES table test from br-3397: starts only after br-3397 merges (dependency edge). Acceptance: just check passes; bridle spec check and coverage clean for the new spec. Model: Sonnet. Out of scope: further specs, and deciding whether the spec flow becomes a rule for bridle's tasks (note the question on the thread when you finish).

## Thread

### note · external:orchestrator · 2026-10-04T18:39:29.252Z
The human, 2026-10-04, via aide (m-4738), choosing option 2 on br-3397 (land as built, file follow-ups): "Agree with 2 - we need to start using specs first bridle".

### note · agent:pm-1 · 2026-10-04T18:39:41.336Z
priority: normal -> high

### note · agent:specs-first · 2026-10-04T20:44:58.608Z
done: design/specs/project-resolution.md + executable scenarios over every command; just check green (1181 tests), spec check/coverage clean; c0a89072a72afd4ba82108a62b8ae90668c5d506

### note · agent:specs-first · 2026-10-04T20:45:00.302Z
Open question (as the ticket asked): should the spec flow (tasks edit design/specs/ in place) become a rule for bridle's own tasks, and which capability is spec'd next? Findings not fixed: migrate and review add|list|remove|resolve act on the cwd repo (SCOPES says Daemon); orchestrator prime succeeds outside a workspace; advisor start outside a workspace prints a command without --project. See the task summary.

### note · agent:specs-first · 2026-10-04T20:50:17.533Z
done: main merged (krz8), just check green (1183 tests); 9ac293e1ba6cd0c8d1a41ed642c507a6b4e5a363

### note · agent:manager-2 · 2026-10-04T20:50:24.966Z
integrated: 2d9adce22f660ebf0add1d610fef22685ee729e7 (branch bridle/specs-first)
