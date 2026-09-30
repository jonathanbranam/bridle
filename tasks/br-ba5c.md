+++
id = "br-ba5c"
title = "Self-upgrade: report the built commit; skip restart for docs-only commits (d3wq)"
kind = "bug"
state = "planned"
created_at = "2026-09-30T12:13:26.538Z"
updated_at = "2026-09-30T12:13:28.830631Z"
+++

Fix docs/tickets/open/self-upgrade-reports-the-wrong-commit-and-restarts-for-docs-d3wq.md, both parts. Files: crates/bridle-daemon upgrade.rs and restart.rs (restart.rs:22 reads the integration ref instead of the built commit, BUILT_KEY). Part 2: skip the upgrade but record the commit as built when git diff --name-only <built>..<candidate> touches nothing the binary is built from; check what include_str!/include_dir! embed (workflow files, docs) before choosing paths, do not guess. Tests: restart message uses built commit; docs-only diff skips, a crates/ or embedded-file change does not. CHANGELOG; docs for self-upgrade if they describe it. Acceptance: just check passes. Model: Sonnet. ORDER: touches the same files as br-4524 (rollback): start after br-4524 lands, or rebase on it. Start-up path: MUST land by Thu 2026-10-01 10:00 ET. Out of scope: rollback, launchd.
