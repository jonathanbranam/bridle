+++
id = "br-b21b"
title = "Upgrade build: 'bridle restart --upgrade' builds verified main then restarts (q7rx 2)"
kind = "feature"
state = "planned"
created_at = "2026-09-30T02:14:07.641Z"
updated_at = "2026-09-30T02:14:09.704888Z"
+++

GOAL (q7rx Shape 1; ticket docs/questions/open/bridle-restarts-itself-q7rx.md). Builds on the restart-in-place task (must land first). 'bridle restart --upgrade' (and the API flag): find the newest commit on main whose CI run on GitHub Actions is green (reuse the daemon's existing CI watcher code, see the failed-run wake; gh or the API), check that commit out/build it with 'cargo install --path crates/bridle' from the clone (background, normal priority), then restart in place. Build only a green main commit; if none is newer than the running binary, say so and do nothing. A failed build leaves the running daemon untouched and reports why (wake to the orchestrator). Tell, don't ask: wake before and after. Tests with faked CI status and a faked build command; docs in step (daemon.md, cli.md, api.md), CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: automatic trigger, other projects' daemons, rollback. Do not run real cargo install in tests.
