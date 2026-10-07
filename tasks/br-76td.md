+++
id = "br-76td"
title = "The gateway records its pid and has stop and restart commands"
kind = "feature"
state = "integrated"
created_at = "2026-10-07T02:11:00.387Z"
updated_at = "2026-10-07T03:23:15.043588Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/gateway-pid-76td"
commit = "ced2f63fdd4f10c522eb0ed7f42cb32b48e6310a"
summary = "The gateway writes ~/.bridle/gateway.pid once listening (PidFile guard in bridle-gateway, removed on drop; SIGTERM/SIGINT now end the select so it drops) and `bridle gateway status|stop|restart` are in crates/bridle/src/gateway.rs. Stop checks `ps -p <pid>` for bridle+gateway, SIGTERMs only that pid via `kill`, waits 10 s, never SIGKILLs. Health now reports `build` (crate version-size-mtime of the gateway's exe); status compares it with the installed binary's for `stale binary`. spawn_detached_with takes explicit args so restart spawns `bridle gateway`. Docs: cli.md, human-web-ui.md, CHANGELOG. Tests in crates/bridle/tests/gateway_test.rs."
+++

original id: 76td
Ticket (the human's words, context; read all of it): docs/tickets/open/the-gateway-records-its-pid-and-has-stop-and-restart-command-76td.md . Related: bek3 (`--detach`), tc7t, rule no-kill-by-name (workflow/base/rules/no-kill-by-name.md: never match processes by pattern; signal only the recorded pid).
Goal: the gateway can be stopped, restarted and inspected by command, with no pid hunting. Auto-restart is NOT wanted yet (the human).
EXACT BEHAVIOUR (names fixed):
- PID FILE: `~/.bridle/gateway.pid`, beside `gateway.log` (use the same home-dir resolver the log path uses, so tests can redirect it). Content: the pid as decimal text plus a newline. Written by the gateway process itself once it is listening (foreground or `--detach`; with `--detach` the child writes it, not the parent), removed on clean exit and on SIGTERM/SIGINT shutdown. A stale file (pid not alive) is ignored and overwritten on the next start. `--detach` keeps refusing when a gateway already answers at `bind`.
- `bridle gateway status`: prints `running`, pid, URL (from `[gateway]` bind/public_url as the existing code derives it) and build (as the gateway's health endpoint reports it; say what you used), plus `stale binary` when the running build differs from the installed `bridle` binary; or `not running` (and `stale pid file removed` if it had to clean one). Exit 0 when running, 1 when not running. Add `--json` as other status commands do.
- `bridle gateway stop`: read the pid file; if absent, say "not running"; verify the pid is alive AND is a bridle gateway by checking that one pid's command line (`ps -p <pid> -o command=`, a specific pid, never a search by name) contains `bridle` and `gateway`; if not, say so, remove nothing, signal nothing. Then send SIGTERM to that pid, wait up to 10 s for it to exit, print `stopped`; if still alive after 10 s, print an error naming the pid and exit 1 (no SIGKILL). Never use a by-name or by-port process search.
- `bridle gateway restart`: stop (as above; not running is fine, not an error), then start exactly like `bridle gateway --detach`, then print the new status line. If stop fails, do not start.
- `bridle gateway --detach` and the existing subcommands (`hash-password`, `install`) are unchanged.
Docs (same change): docs/design/cli.md gateway lines (~53-60), docs/design/human-web-ui.md "Running it detached" (pid file, stop, restart, status), CHANGELOG entry (read with a limit). Files likely: crates/bridle-gateway (startup, shutdown, pid file), crates/bridle/src (the gateway subcommands in cli.rs and its command module).
Tests (localhost only; never signal a process you did not start): pid file written on start and removed on clean shutdown; stale pid file ignored; `stop` refuses when the recorded pid's command is not a gateway (use a harmless child you spawned, e.g. `sleep`, as the wrong process, and assert it is still alive afterwards); `stop` ends a real test gateway you started and removes the file; status output for running and not running; restart gives a new pid.
Acceptance: just check passes. Migration: none (new file, new commands). Model: Sonnet. Out of scope: auto-restart, launchd/systemd changes, the daemon's own stop/restart, any other gateway behaviour. Run it AFTER br-vn42 (the human wants the upgrade work first).

## Thread

### note · external:orchestrator · 2026-10-07T02:11:23.283Z
From orchestrator: br-76td (gateway pid file, bridle gateway stop/restart/status) is the human's ask, relayed by aide; quote on ticket 76td. Ready now; plan it after br-vn42 (normal priority, not urgent). Small, likely Sonnet; follow rule no-kill-by-name (stop only the recorded pid).

### note · agent:gateway-pid-76td · 2026-10-07T03:22:59.026Z
done: pid file, gateway status/stop/restart; build = health's new build field (version-size-mtime). just check exit 0, 1278 tests passed (last full: 1266), commit 09bc044d.

### note · agent:gateway-pid-76td · 2026-10-07T03:23:01.378Z
done: gateway pid file + status/stop/restart; just check exit 0, 1278 tests; 09bc044d

### note · agent:manager-2 · 2026-10-07T03:23:06.440Z
integrated: ced2f63fdd4f10c522eb0ed7f42cb32b48e6310a (branch bridle/gateway-pid-76td)

### note · agent:manager-2 · 2026-10-07T03:23:15.043Z
cleanup: removed agent gateway-pid-76td, branch bridle/gateway-pid-76td
