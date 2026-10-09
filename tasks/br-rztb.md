+++
id = "br-rztb"
title = "Incident: something keeps restarting dalek's gateway outside launchd from a Claude session (pid 88281 since 10-08 9:39 PM); recurring"
kind = "incident"
state = "planned"
created_at = "2026-10-09T19:18:13.267Z"
updated_at = "2026-10-09T19:36:53.912558Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
priority_at = "2026-10-09T19:18:48.930751Z"
ticket = "rztb"
+++

Ticket: docs/tickets/open/incident-something-keeps-restarting-dalek-s-gateway-outside-rztb.md (read it: the human's words, the facts, the 4 asks; also br-57nt, ppa6, 76td, bek3, and docs/context/incidents.md). Model: Sonnet. Investigate first, then fix.

Do, in order:
1. Find the caller. Read where the gateway is started and restarted: the `bridle gateway` CLI (start, --detach, restart, stop; crates/bridle/src and crates/bridle-gateway), the gateway's self re-exec on a changed binary ("the bridle binary changed; restarting onto it"), `just install` and the daemon's self-upgrade (grep justfile and crates/bridle-daemon for gateway), launchd plist/units, and every test that runs `bridle gateway` (including br-57nt's). Answer from the code and the logs (~/.bridle/gateway.log, `launchctl print` output is read-only and allowed; do NOT start, stop, kickstart or restart anything live, and do not touch pid 88281). Report on the task thread which code paths can start a gateway outside launchd and which most likely did at 2026-10-08 9:39:49 PM ET (01:39:49Z). Say plainly what is proven and what is inferred.
2. Fix: nothing but launchd (or systemd) starts or restarts the LIVE gateway on a machine that has the unit installed: `gateway restart`/`--detach` against a launchd-managed gateway goes through launchd (kickstart) or refuses; a self re-exec onto a new binary keeps launchd supervision (exit and let launchd restart, if the unit is installed) rather than re-executing in place with the caller's env. Check the Claude env (CLAUDECODE, CLAUDE_CODE_ENTRYPOINT, BRIDLE_AS) never ends up in a gateway. Tests must never touch the real gateway, its pid file (~/.bridle/gateway.pid) or launchd: add a guard/test that proves test runs use a temp home and no real launchctl.
3. Log who asked: every gateway stop/start/restart writes a line to gateway.log with the caller (BRIDLE_AGENT_NAME / BRIDLE_AS / user, parent pid and its command, and whether it came via launchd, the CLI, or self re-exec).
4. Docs: docs/design (gateway doc, grep for gateway.pid), docs/context/incidents.md entry updated with the cause, CHANGELOG.md.
Acceptance: just check passes; findings on the thread. Migration: none for project files; the launchd unit (if you change it) is reinstalled by the existing `bridle gateway install`; state that.
Out of scope: stopping pid 88281 and handing the port back to launchd (step 4 of the ticket; needs the human's OK; the orchestrator does it), the systemd uninstall (br-xrkh), anything about the daemon (not the gateway).

## Thread

### note · external:orchestrator · 2026-10-09T19:18:48.930Z
priority: normal -> high

### note · external:orchestrator · 2026-10-09T19:18:49.040Z
orchestrator: readied, high (the human, via aide, verbatim on the ticket: 'This needs to be investigated; I have reported this before'). Investigation first: find the caller, check whether any test (br-57nt's included) touches the real gateway or launchd. Step 4 (stopping pid 88281) needs the human's OK. Logged in docs/context/incidents.md.

### note · external:orchestrator · 2026-10-09T19:18:49.191Z
From orchestrator: br-rztb (gateway restarted outside launchd, recurring, human-reported) is ready, high. Plan it next after work already claimed; Sonnet. See thread.

### note · external:orchestrator · 2026-10-09T19:31:00.316Z
From orchestrator: br-rztb planned. Queue it at the front.

### note · external:aide · 2026-10-09T19:31:13.539Z
From the aide, 3:31 PM ET: at 3:17:14 PM ET pid 88281 got SIGTERM (not from the aide; sender unknown) and launchd's dev.bridle.gateway took the port: now pid 97811, state running, XPC_SERVICE_NAME=dev.bridle.gateway, no CLAUDECODE. The 3:30 PM daemon upgrade (355636740) did not touch it. The investigation still stands: what started 88281 and the earlier restarts.

### note · external:orchestrator · 2026-10-09T19:31:23.616Z
From orchestrator: br-rztb (incident, high, planned; brief in the task) can't be queued (incidents aren't). Claim it directly for the next free worker slot, ahead of queue work. Sonnet. Stay at two workers.

### note · external:orchestrator · 2026-10-09T19:36:53.912Z
From orchestrator (repeat after your renewal): br-rztb (incident, high, planned) takes the next free worker slot, Sonnet, ahead of queue work. Incidents can't be queued, so claim it directly.
