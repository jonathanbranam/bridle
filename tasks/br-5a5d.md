+++
id = "br-5a5d"
title = "bridle doctor: check the project's setup and say what to fix"
kind = "feature"
state = "planned"
created_at = "2026-09-29T09:35:40.702Z"
updated_at = "2026-09-29T09:35:42.365774Z"
+++

Goal (docs/design/cli.md Planned: 'bridle init | doctor'; onboarding friction for new projects): `bridle doctor [--json]`, local first, daemon checks only if one is running. Checks, each printing ok / warn / fail with a one-line fix: git repo and integration branch exists ([branches] integration; the g3ck failure mode); `claude` on PATH and version; `gh` on PATH when [ci] is enabled; .bridle/config.toml parses (reuse config.rs loading and its error text) and referenced files exist (system_prompt paths, workflow root, packs, components docs); `bridle sync` would change nothing (dry: use the existing sync in check mode if there is one, else skip); .gitignore covers .bridle/cache/ and the runtime files bridle writes (bridle.db, daemon.json); state branch present if tasks exist; every declared role has a prompt; port/reserved config sane; git >= 2.38 (merge-tree). Exit 1 if any fail. Files: crates/bridle (new doctor.rs, cli.rs), reuse crates/bridle-daemon config/rules/sync; docs cli.md, and a short 'Onboarding a project' pointer in docs/README.md if there is a natural place.

Acceptance: just check passes; tests on a temp repo: healthy config passes, missing integration branch fails with the fix line, bad config reports the parse error. Model: Sonnet. Out of scope: fixing problems automatically (init does scaffolding), any change in real projects.
