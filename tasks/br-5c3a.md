+++
id = "br-5c3a"
title = "NUC B2: 'bridle prime orchestrator' and advisor work for a project other than bridle"
kind = "feature"
state = "planned"
created_at = "2026-09-30T03:01:34.280Z"
updated_at = "2026-09-30T03:33:24.287661Z"
summary = "Orchestrator role split: workflow/base/roles/orchestrator.md is generic ({project} substituted by prime with the --project/dir name, incl. the credentials snippet); prime appends optional <repo>/.bridle/roles/orchestrator.md; state file now optional; bridle's specifics moved to .bridle/roles/orchestrator.md. Advisor role generic, optional .bridle/roles/advisor.md (bridle's in place). Startup steps de-bridled. Tests, cli.md, CHANGELOG updated. Scripts untouched."
+++

GOAL: 'bridle prime orchestrator' (and advisor) must work for a project other than bridle, e.g. meta-notes served by its own daemon on the NUC. Today workflow/base/roles/orchestrator.md assumes bridle: two managers, GitHub CI, cargo install, role-notes, a credentials entry named 'bridle'. Do: (1) split into a generic orchestrator role (workflow/base/roles/orchestrator.md: works with one manager or whatever agents the project's daemon has, no bridle-repo specifics) plus an optional per-project part loaded from <repo>/.bridle/roles/orchestrator.md when present, appended by prime; move the bridle-specific text into bridle's own .bridle/roles/orchestrator.md (bridle repo). (2) the token snippet and any 'bridle' credentials-entry text use the current project's name. (3) the handover note and state come from that project's daemon; the docs/context/orchestrator-state.md fallback is optional (skip if not trivial). (4) the advisor role: files tickets for the project it serves (generic wording, project part optional the same way). Code: crates/bridle/src/prime.rs (and ORCHESTRATOR_STARTUP_STEPS in commands.rs). Tests: prime output for a temp project without the per-project file has no bridle-specific strings (cargo install, role-notes); with the file, it is appended; token snippet uses the project name. Docs in step, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: the scripts (B1), NUC setup, workflow-path resolution (NUC A; run after it, same files).

## Thread

### note · agent:prime-generic · 2026-09-30T03:33:24.287Z
done: generic orchestrator/advisor roles + optional per-project parts, prime uses project name; just check green (790 tests); c7b22b5
