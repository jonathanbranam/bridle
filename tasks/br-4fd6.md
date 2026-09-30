+++
id = "br-4fd6"
title = "NUC A: workflow path per machine (expand ~/$VAR, ~/.bridle/config.toml override, loud on missing)"
kind = "bug"
state = "planned"
created_at = "2026-09-30T03:01:34.219Z"
updated_at = "2026-09-30T03:01:35.553178Z"
+++

GOAL: a project's .bridle/config.toml 'workflow' path (meta-notes on bridle-adopt has workflow = /Volumes/Data/work/bridle/bridle/workflow) must work on another machine, e.g. the NUC (Ubuntu). Today a missing dir silently drops base rules and bridle sync installs no bridle-worker skill. Do: (1) expand ~ and $VARs in the workflow value; (2) let ~/.bridle/config.toml override it per machine (machine layer beats project; document precedence in docs/design/ config docs); (3) a missing or unreadable workflow dir is a loud error at serve/sync and a failing check in 'bridle doctor', never silent. Readers to update, one shared resolver preferred: crates/bridle-daemon/src/config.rs (default_role_prompts), rules.rs (discover_layers), doctor.rs, prime.rs, crates/bridle/src/commands.rs. Tests: expansion, override precedence, missing dir errors, doctor fails. Docs in step, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: NUC setup itself, changing meta-notes' config. Touches prime.rs and config.rs: run before NUC B2.
