+++
id = "br-9a7b"
title = "daemon restart --upgrade ignores self_upgrade = release and looks for a green commit"
kind = "bug"
state = "planned"
created_at = "2026-10-15T21:01:32.908Z"
updated_at = "2026-10-15T22:07:08.635965Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "9a7b"
+++

docs/tickets/open/daemon-restart-upgrade-ignores-self-upgrade-release-and-look-9a7b.md

## Thread

### note · external:advisor/product-manager · 2026-10-15T21:56:05.711Z
Readied by the human via advisor product-manager, 2026-10-15 ~5:20 PM ET: "Yes, ready both".

### note · external:advisor/product-manager · 2026-10-15T21:56:19.491Z
Correction: readied ~5:55 PM ET, not ~5:20.

### note · agent:pm-1 · 2026-10-15T21:56:52.796Z
Brief (pm-1). Model: Sonnet. Size: S/M. Fix per the ticket body: in crates/bridle-daemon/src/server.rs (~4718, the `req.upgrade` branch of the restart handler) honour `SelfUpgrade::Release`: look up the newest release on `release_repo` newer than the running binary using the same lookup the automatic upgrade path uses (find it with git grep -n "SelfUpgrade::Release" crates/; reuse it, do not write a second lookup), and install it as the automatic path does. The "nothing to upgrade" text names releases in release mode, green commits in commit mode. Tests for both modes (fake the release lookup the way the automatic path's tests do). Docs: docs/design/agent-host (restart/upgrade) and CHANGELOG. Acceptance: just check. Related br-mtdg (the manual upgrade builds the project's repo instead of bridle's, same handler): check its state with bridle task show br-mtdg; if mtdg is open or planned, tell the manager they touch the same branch so they run one after the other; do not fix mtdg here. Out of scope: commit-mode behaviour changes. No migration.
