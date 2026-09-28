+++
id = "br-e8b3"
title = "Clean stale build output in the main clone (f75x)"
kind = "chore"
state = "planned"
created_at = "2026-09-28T15:37:14.739Z"
updated_at = "2026-09-28T16:28:36.479135Z"
+++

ticket: docs/questions/open/clean-stale-build-output-f75x.md
original id: f75x

Low priority, after P2 (the human).

target/ in the long-lived main clone grows without bound (28G observed: 21G target/debug/deps -- ~186,000 stale .o files from old hashes each rebuild -- 7G target/debug/incremental). Worker worktrees already get cleaned up via `bridle rm --delete-branch`; this is only about the main clone, which is rebuilt after nearly every merge by the manager/orchestrator`just check`.

Add periodic cleanup: either `cargo clean` as part of whatever the orchestrator role already does at release time (.bridle/roles/orchestrator.md), or `cargo sweep` to drop artifacts past some age. Must not run concurrently with a `just check` build in the same clone -- figure out how to serialize or guard against that (e.g. only run when no check is in flight, or as a step inside the same sequenced process that runs checks).

See nbkj (smaller debug profile) and m3wq (disk monitoring) -- related but independent.

Acceptance: just check passes; document where/how often the cleanup runs.
