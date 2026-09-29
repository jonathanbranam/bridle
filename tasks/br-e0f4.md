+++
id = "br-e0f4"
title = "bridle init: scaffold .bridle for a project"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T09:35:40.716Z"
updated_at = "2026-09-29T10:40:06.226982Z"
branch = "bridle/init-cmd"
commit = "2780c95a516d16bfbc5703617e30944acf8dd221"
summary = "Added 'bridle init' (crates/bridle/src/init.rs, cli.rs): scaffolds .bridle/config.toml (integration from HEAD or --integration, workflow only if repo has workflow/base else commented, packs from --stack, check command detected from justfile/Cargo/package.json/pyproject, worktree stubs) and appends .gitignore lines; never overwrites, lists skipped files. --name is comment-only since config has no name key. Tests on temp repos run doctor's local_checks on the result. Docs: cli.md, README onboarding pointer, CHANGELOG."
+++

Goal (cli.md Planned; onboarding): `bridle init [--name N] [--integration BRANCH] [--stack python|typescript|rust]` in a git repo creates, only if absent (never overwrites; existing files are listed as skipped): .bridle/config.toml with commented sections for [project], [branches] integration (detected from HEAD's branch or --integration), roles (manager, worker with the base role prompts via the default path from br-f636), [worktrees] setup/copy stubs, [integration] check stub, packs = [stack] when --stack is given; .gitignore lines for .bridle/cache/, bridle.db and the runtime files (append, dedupe); prints next steps (run bridle sync, bridle doctor, bridle serve). Reads the values it can from the repo: default branch, presence of justfile/package.json/Cargo.toml/pyproject.toml to suggest --stack and the check command. It does NOT run sync or start anything. Files: crates/bridle (new init.rs, cli.rs), docs cli.md, docs/README.md onboarding pointer. Look at bridle's own .bridle/config.toml and the recent onboarding docs in docs/context/ for what real projects ended up needing.

Acceptance: just check passes; tests on temp repos: fresh init creates the files and `bridle doctor` (if merged; else the config loader) accepts the result; rerun changes nothing; existing config is never touched. Model: Sonnet. Out of scope: touching any real project, prompts or interactive questions. Run after the doctor task (shared cli.rs).

## Thread

### note · agent:manager-2 · 2026-09-29T10:40:06.226Z
integrated: 2780c95a516d16bfbc5703617e30944acf8dd221 (branch bridle/init-cmd)
