+++
id = "br-b21b"
title = "Upgrade build: 'bridle restart --upgrade' builds verified main then restarts (q7rx 2)"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T02:14:07.641Z"
updated_at = "2026-09-30T02:38:02.739369Z"
branch = "bridle/upgrade-build"
commit = "c89909e0eb66f4b2c912beacb244e7cf914ae019"
summary = """`bridle restart --upgrade` / `upgrade: true` on POST /v1/restart. New upgrade.rs: newest first-parent commit on main (30 back) whose GH runs (the CI watcher's Gh trait, injectable via Overrides.upgrade) are all green; `meta upgrade.built` records the last built commit (binary has no embedded commit), candidate equal/ancestor of it = "nothing to upgrade". Otherwise reply at once and background: wake, build in a throwaway detached worktree (<ws>/.bridle/upgrade-src, CARGO_TARGET_DIR=<ws>/.bridle/upgrade-target kept for incremental builds; human's checkout untouched), then the existing restart path (refactored into perform_restart). Build failure or no quiet point: daemon untouched, wake upgrade_failed + human inbox note. RestartResponse gained restarting/message. Tests in upgrade_test.rs with fake gh and fake build command. Docs and CHANGELOG updated. Caveat: if a human cargo-installs by hand, upgrade.built is stale."""
+++

GOAL (q7rx Shape 1; ticket docs/questions/open/bridle-restarts-itself-q7rx.md). Builds on the restart-in-place task (must land first). 'bridle restart --upgrade' (and the API flag): find the newest commit on main whose CI run on GitHub Actions is green (reuse the daemon's existing CI watcher code, see the failed-run wake; gh or the API), check that commit out/build it with 'cargo install --path crates/bridle' from the clone (background, normal priority), then restart in place. Build only a green main commit; if none is newer than the running binary, say so and do nothing. A failed build leaves the running daemon untouched and reports why (wake to the orchestrator). Tell, don't ask: wake before and after. Tests with faked CI status and a faked build command; docs in step (daemon.md, cli.md, api.md), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: automatic trigger, other projects' daemons, rollback. Do not run real cargo install in tests.

## Thread

### note · agent:upgrade-build · 2026-09-30T02:37:36.892Z
done: bridle restart --upgrade (API upgrade flag): newest green-CI main commit, built in a throwaway worktree, then restart in place; nothing newer says so; build failure leaves daemon up and wakes orchestrator + human note. just check green (780 tests), main merged; 7df17f5

### note · agent:manager-2 · 2026-09-30T02:37:43.060Z
main moved. Merge main, confirm ancestor (just check if the merge touched code), message me the sha.

### note · agent:upgrade-build · 2026-09-30T02:37:48.064Z
merged main again (docs-only change, no code, so no re-check); main is an ancestor, br-2b1b 727184b included; sha 3d4c6a4

### note · agent:manager-2 · 2026-09-30T02:37:50.722Z
integrated: c89909e0eb66f4b2c912beacb244e7cf914ae019 (branch bridle/upgrade-build)

### note · agent:manager-2 · 2026-09-30T02:38:02.739Z
cleanup: removed agent upgrade-build, branch bridle/upgrade-build
