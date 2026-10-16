---
id: br-9a7b
title: daemon restart --upgrade ignores self_upgrade = release and looks for a green commit
kind: bug
created: 2026-10-15
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-mtdg, br-fv86]
resolved: 2026-10-16T10:02:58Z
---

## The ask

Reported by orchestrator@vesta, 2026-10-15 15:31 UTC (message m-13526).

Vesta's `~/.bridle/config.toml` has `[daemon] self_upgrade = "release"` and
`release_repo = "jonathanbranam/bridle"`, running 0.7.3. GitHub release v0.7.4 was Latest
(published 15:30 UTC, linux tarball + SHA256SUMS, CI green). Two tries of
`bridle daemon restart --upgrade --project notes` (15:31, 15:33 UTC) both answered:

    nothing to upgrade to: no commit on main with green CI is newer than the running binary

The daemons then self-upgraded to 0.7.4 on their own at 15:33 UTC, so the automatic path
honours release mode and only the manual one doesn't.

Cause (read in code, 2026-10-15): the restart handler in `crates/bridle-daemon/src/server.rs`
(around line 4718, the `req.upgrade` branch) always calls `state.upgrader.newest_green(...)`
on the project's repo, the commit-mode lookup, and never checks `SelfUpgrade::Release`.

Fix: when `self_upgrade = "release"`, `restart --upgrade` looks up the newest release on
`release_repo` newer than the running binary (the same lookup the automatic upgrade uses) and
installs it; the "nothing to upgrade" text names releases, not green commits. Test both modes.
Related: mtdg (the manual upgrade builds the project's repo instead of bridle's: the same
handler, same lookup).
