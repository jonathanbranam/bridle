+++
id = "br-ba5c"
title = "Self-upgrade: report the built commit; skip restart for docs-only commits (d3wq)"
kind = "bug"
state = "integrated"
created_at = "2026-09-30T12:13:26.538Z"
updated_at = "2026-09-30T12:48:38.700281Z"
branch = "bridle/upgrade-commit"
commit = "e682bd74c71e943aa451bb174a74f5ed64486db0"
summary = """Self-upgrade now reports the commit it built (restart::running_commit reads upgrade.built, falling back to the head before any upgrade) in the agents' note and the restart wakes. upgrade_in_background first calls upgrade::needs_build: git diff --name-only <built>..<sha> touching nothing under crates/, .cargo/, Cargo.toml, Cargo.lock or rust-toolchain* records the commit as built, wakes "skipped" and doesn't restart; no built commit or a git failure builds. Every include_str!/include_bytes! in the crates is in test code, so workflow/ and docs/ are not build inputs. Tests in upgrade.rs and restart.rs; daemon.md and CHANGELOG updated; ticket d3wq moved to resolved. Caveat: bridle-spec parses_fast (1s wall-clock limit) fails under full-suite load here (passes alone); unrelated to this change."""
+++

Fix docs/tickets/open/self-upgrade-reports-the-wrong-commit-and-restarts-for-docs-d3wq.md, both parts. Files: crates/bridle-daemon upgrade.rs and restart.rs (restart.rs:22 reads the integration ref instead of the built commit, BUILT_KEY). Part 2: skip the upgrade but record the commit as built when git diff --name-only <built>..<candidate> touches nothing the binary is built from; check what include_str!/include_dir! embed (workflow files, docs) before choosing paths, do not guess. Tests: restart message uses built commit; docs-only diff skips, a crates/ or embedded-file change does not. CHANGELOG; docs for self-upgrade if they describe it. Acceptance: just check passes. Model: Sonnet. ORDER: touches the same files as br-4524 (rollback): start after br-4524 lands, or rebase on it. Start-up path: MUST land by Thu 2026-10-01 10:00 ET. Out of scope: rollback, launchd.

## Thread

### note · agent:upgrade-commit · 2026-09-30T12:47:14.674Z
done: restart messages use the built commit; docs-only diffs skip the build (recorded as built). Merged main, committed f6a008d. CAVEAT: just check exits 100 only on bridle-spec parses_fast (1s wall-clock limit, ~1.1-1.2s under full-suite load; passes alone in 0.2s); all else passes (885/886). Unrelated to this change, likely flaky on loaded host.

### note · agent:manager-2 · 2026-09-30T12:47:19.185Z
Looks good. main moved: merge main into your branch and message me right away. If just check fails only on bridle-spec parses_fast under load, rerun that test alone and tell me; that's the known flake.

### note · agent:upgrade-commit · 2026-09-30T12:48:26.507Z
main merged again, head 1318552. just check: 885/886, only parses_fast failed under load; rerun alone passes (known flake).

### note · agent:manager-2 · 2026-09-30T12:48:28.891Z
integrated: e682bd74c71e943aa451bb174a74f5ed64486db0 (branch bridle/upgrade-commit)

### note · agent:manager-2 · 2026-09-30T12:48:38.700Z
cleanup: removed agent upgrade-commit, branch bridle/upgrade-commit
