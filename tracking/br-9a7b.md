+++
id = "br-9a7b"
title = "daemon restart --upgrade ignores self_upgrade = release and looks for a green commit"
kind = "bug"
state = "integrated"
created_at = "2026-10-15T21:01:32.908Z"
updated_at = "2026-10-16T00:10:56.679188Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:9a7b",
]
branch = "bridle/9a7b"
commit = "0edb5586129f2801fdff2931e5c391d5b69b1aaa"
summary = "Fix: restart --upgrade under self_upgrade = release now uses the same release lookup as the automatic upgrade (newest_release in server.rs, shared by both) and starts release_upgrade_in_background; the nothing-to-upgrade text names releases. Commit mode unchanged. Tests in upgrade_test.rs; docs: daemon.md, cli.md, CHANGELOG. br-mtdg (manual upgrade builds the project's repo) has no row on this daemon, so I could not check its state; it touches the same commit-mode handler lines."
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

### note · agent:9a7b · 2026-10-15T23:26:49.404Z
worker 9a7b: code, tests, docs, CHANGELOG, summary done and main merged; waiting on the final just check (log /tmp/9a7b-check.log), then commit and report sha to manager-2.

### note · agent:9a7b · 2026-10-15T23:43:47.279Z
done: restart --upgrade honours self_upgrade=release (shared newest_release lookup), tests both paths, docs+CHANGELOG; just check exit 0, 1719 tests passed (main already contained); 0218dc54df3bebc19fc359a5add98006c0595024. br-mtdg has no row on my daemon, so unchecked; it touches the same handler's commit-mode lines.

### note · agent:manager-2 · 2026-10-15T23:43:53.368Z
built: 0218dc54df3bebc19fc359a5add98006c0595024 (branch bridle/9a7b)

### note · agent:manager-2 · 2026-10-16T00:10:39.853Z
integrated: 0edb5586129f2801fdff2931e5c391d5b69b1aaa (branch bridle/9a7b)

### note · agent:manager-2 · 2026-10-16T00:10:56.679Z
cleanup: removed agent 9a7b, branch bridle/9a7b
