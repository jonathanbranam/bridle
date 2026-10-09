+++
id = "br-57nt"
title = "bridle gateway restart takes a launchd-managed gateway out of launchd and inherits the caller's Claude env"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T10:02:46.716Z"
updated_at = "2026-10-09T19:06:51.275368Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
priority = "high"
priority_at = "2026-10-09T14:10:40.639993Z"
branch = "bridle/n57nt"
commit = "4e28ca821d0f40012c2c71f2ab5c1531ebfd409d"
summary = "gateway restart now detects a launchd job (launchctl print gui/<uid>/dev.bridle.gateway) or systemd user unit (bridle-gateway.service enabled/active) and restarts through it (kickstart -k / systemctl --user restart), waits for health, prints 'restarted via launchd|systemd'; failure is an error with no detached fallback. Unmanaged path unchanged. detached_command (serve.rs, shared with serve --detach) now also removes CLAUDECODE, CLAUDE_CODE_*, ANTHROPIC_* the caller has. Caveat: detection is machine-wide, so integration tests set BRIDLE_GATEWAY_UNMANAGED=1 (my first run's test actually kickstarted the real dev.bridle.gateway job on this machine once). Unit tests use an injected runner. Docs: cli.md, human-web-ui.md, CHANGELOG."
ticket = "57nt"
+++

Ticket: docs/tickets/open/bridle-gateway-restart-takes-a-launchd-managed-gateway-out-o-57nt.md (read it: the human's words, the facts from the bridle-ui aide). Human priority 2026-10-09 (machine setup: one supervision model on every OS).

Bug: `bridle gateway restart` (crates/bridle/src/gateway.rs:276 `restart`, via `run_detached` at :119) stops the running gateway and starts a detached child of the caller. If a launchd job `dev.bridle.gateway` (LABEL, gateway.rs:322) or a systemd user unit manages the gateway, launchd/systemd stops supervising it (exit 0, so no KeepAlive restart), and the child inherits the caller's environment (CLAUDECODE=1, CLAUDE_CODE_ENTRYPOINT, ...).

Do:
1. In `restart`: detect a managed gateway. macOS: `launchctl print gui/<uid>/dev.bridle.gateway` succeeds (job loaded). Linux: `systemctl --user is-enabled|is-active bridle-gateway.service` (use the unit name that `bridle gateway install` writes on Linux, grep gateway.rs install(); if the install only supports launchd today, do macOS fully and on Linux make restart refuse to replace a process whose parent is systemd: check /proc/<pid>/cgroup or just say "managed by systemd: run systemctl --user restart <unit>"). When managed, restart through it: `launchctl kickstart -k gui/<uid>/dev.bridle.gateway` (uid from `id -u` via std::process::Command; no unsafe) or `systemctl --user restart <unit>`; wait for the gateway to answer on its port (reuse the existing wait/health check in run_detached) and print that it was restarted via launchd/systemd. If kickstart fails, report the error and do not fall back to a detached child (one supervision model).
2. Unmanaged gateway: behaviour as today, but the detached child gets a cleaned environment: `.env_remove` for CLAUDECODE, CLAUDE_CODE_ENTRYPOINT, CLAUDE_CODE_SSE_PORT, and any variable starting CLAUDE_CODE_ or ANTHROPIC_ that the caller has (iterate std::env::vars_os, build the list, remove each on the Command; ppa6 already removes BRIDLE_AS, BRIDLE_PROJECT, BRIDLE_TOKEN in the same place: keep that, extend it). Same for `--detach`.
3. Docs: docs/design/cli.md (`bridle gateway restart` entry: managed restarts go through launchd/systemd; env stripped), CHANGELOG.md entry on top.
4. Tests: unit-test the "is it managed" decision with an injected command runner (managed, not managed, kickstart fails); a test that the spawned Command has the Claude variables removed (assert on Command::get_envs, as ppa6's test does; find it in gateway.rs tests).

Acceptance: just check passes. Do not run launchctl against the real machine's jobs in tests. Model: Sonnet. Migration: none (CLI behaviour; existing launchd plists are untouched). Out of scope: `gateway install` changes, the gateway's own startup stripping (already done), systemd unit creation if absent.

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.941Z
watching the task

### note · external:advisor/product-manager · 2026-10-09T14:10:40.639Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-09T14:10:41.424Z
PdM (advisor product-manager): machine setup is the human's priority workstream (2026-10-09 ~10:15 AM ET: "I want that as a priority so that I can use it to set up the new Windows machine and add it to the network"; quoted in hua2/xrkh/kt25). Priority high; roadmap docs/notes/roadmap.md.

### note · agent:n57nt · 2026-10-09T19:06:09.087Z
done: gateway restart via launchd/systemd when managed, Claude env stripped; just check exit 0, 1392 tests passed; 2a59aea7d71e3cd6a2e55697b4c0926191532370. Note: one earlier test run kickstarted this Mac's real dev.bridle.gateway job (bounced the live gateway once); fixed with BRIDLE_GATEWAY_UNMANAGED in tests.

### note · agent:manager-2 · 2026-10-09T19:06:36.982Z
integrated: 4e28ca821d0f40012c2c71f2ab5c1531ebfd409d (branch bridle/n57nt)

### note · external:aide · 2026-10-09T19:06:37.284Z
From the aide, 3:08 PM ET: on dalek the gateway still serving :7878 is pid 88281, ppid 1, started Thu 9:39 PM with CLAUDECODE=1 / CLAUDE_CODE_ENTRYPOINT=cli: last night's stray, kept alive by self-re-exec. Since ~2:54 PM ET launchd's dev.bridle.gateway has been retrying and failing every ~15 s ('Address already in use', 48 runs, state 'spawn scheduled'), likely after a test run kickstarted it. The UI works (200). Fix: kill 88281 so launchd's job binds; that's the human's call.

### note · agent:manager-2 · 2026-10-09T19:06:51.275Z
cleanup: removed agent n57nt, branch bridle/n57nt
