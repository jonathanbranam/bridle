+++
id = "br-4fd6"
title = "NUC A: workflow path per machine (expand ~/$VAR, ~/.bridle/config.toml override, loud on missing)"
kind = "bug"
state = "integrated"
created_at = "2026-09-30T03:01:34.219Z"
updated_at = "2026-09-30T03:13:22.284404Z"
branch = "bridle/workflow-path"
commit = "6b0d0388e1d299e6f606417314ccb6bcd722dfe1"
summary = "The project's workflow path now expands ~ and $VAR/${VAR} (unset var is an error), and ~/.bridle/config.toml's workflow overrides the project's (machine beats project). Config::workflow_root is the shared resolver: a missing or unreadable dir is an error at serve, sync, prime and rules, and fails doctor's 'referenced files' check. The orchestrator prime now reads its role file from the resolved workflow. Docs in roles-and-config.md, CHANGELOG. Git-url workflows are still unresolved."
+++

GOAL: a project's .bridle/config.toml 'workflow' path (meta-notes on bridle-adopt has workflow = /Volumes/Data/work/bridle/bridle/workflow) must work on another machine, e.g. the NUC (Ubuntu). Today a missing dir silently drops base rules and bridle sync installs no bridle-worker skill. Do: (1) expand ~ and $VARs in the workflow value; (2) let ~/.bridle/config.toml override it per machine (machine layer beats project; document precedence in docs/design/ config docs); (3) a missing or unreadable workflow dir is a loud error at serve/sync and a failing check in 'bridle doctor', never silent. Readers to update, one shared resolver preferred: crates/bridle-daemon/src/config.rs (default_role_prompts), rules.rs (discover_layers), doctor.rs, prime.rs, crates/bridle/src/commands.rs. Tests: expansion, override precedence, missing dir errors, doctor fails. Docs in step, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: NUC setup itself, changing meta-notes' config. Touches prime.rs and config.rs: run before NUC B2.

## Thread

### note · agent:workflow-path · 2026-09-30T03:13:17.562Z
done: workflow path expands ~/$VAR, machine config overrides project, missing dir is loud at serve/sync/prime/rules and fails doctor (shared Config::workflow_root); just check green (788 tests), main merged; 2e62f7a

### note · agent:manager-2 · 2026-09-30T03:13:22.284Z
integrated: 6b0d0388e1d299e6f606417314ccb6bcd722dfe1 (branch bridle/workflow-path)
