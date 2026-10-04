+++
id = "br-qpr7"
title = "bridle uses its own specs, starting with how every command finds its project"
kind = "feature"
state = "planned"
created_at = "2026-10-04T18:39:04.274Z"
updated_at = "2026-10-04T18:39:41.336610Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
+++

original id: qpr7
Build docs/tickets/open/bridle-uses-its-own-specs-starting-with-how-every-command-fi-qpr7.md (read it, plus docs/design/specs.md, specs-to-tests.md, spec-flow.md 'Adopting, in order'). Bridle adopts its own specs: create design/specs/ for bridle, with a first spec for how every project-scoped command finds its project (--project, then BRIDLE_PROJECT, then the cwd's workspace, refuse otherwise, never a hard-coded bridle). Add an EXECUTABLE scenario that runs every project-scoped command, subcommands included, from a temp workspace for project X with no --project and checks it resolved X (or refused outside any workspace); link spec to tests so bridle spec check and bridle spec coverage pass. Builds on crates/bridle/src/project.rs and the SCOPES table test from br-3397: starts only after br-3397 merges (dependency edge). Acceptance: just check passes; bridle spec check and coverage clean for the new spec. Model: Sonnet. Out of scope: further specs, and deciding whether the spec flow becomes a rule for bridle's tasks (note the question on the thread when you finish).

## Thread

### note · external:orchestrator · 2026-10-04T18:39:29.252Z
The human, 2026-10-04, via aide (m-4738), choosing option 2 on br-3397 (land as built, file follow-ups): "Agree with 2 - we need to start using specs first bridle".

### note · agent:pm-1 · 2026-10-04T18:39:41.336Z
priority: normal -> high
