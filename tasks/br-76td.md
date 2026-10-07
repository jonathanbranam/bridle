+++
id = "br-76td"
title = "The gateway records its pid and has stop and restart commands"
kind = "feature"
state = "planned"
created_at = "2026-10-07T02:11:00.387Z"
updated_at = "2026-10-07T02:11:47.714319Z"
created_by = "external:aide"
watchers = ["external:aide"]
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
