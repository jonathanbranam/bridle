+++
id = "br-rztb"
title = "Incident: something keeps restarting dalek's gateway outside launchd from a Claude session (pid 88281 since 10-08 9:39 PM); recurring"
kind = "incident"
state = "planned"
created_at = "2026-10-09T19:18:13.267Z"
updated_at = "2026-10-09T20:28:28.899565Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
priority_at = "2026-10-09T19:18:48.930751Z"
summary = "Cause: pre-57nt `gateway restart`/`--detach` started a detached gateway outside launchd with the caller's env, and such a gateway re-exec'd in place; callers were never logged (01:39Z caller itself unknowable). Fix in crates/bridle/src/gateway.rs (+ serve.rs detached_command shared): --detach refused when a unit is loaded; unit-child gateway exits non-zero on binary change for launchd/systemd to restart; unmanaged re-exec scrubs Claude/principal env; log_caller writes a line to gateway.log for stop/start/restart. Tests (gateway_test.rs) shim launchctl/systemctl on PATH and assert never called, temp-home assertion, caller-log and detach-refusal tests. Docs: human-web-ui.md, cli.md, incidents.md, CHANGELOG. Plist unchanged, no reinstall. just check passed (1398 tests); main merged afterwards (docs/tickets only)."
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

### note · agent:pair8c25 · 2026-10-09T20:03:00.068Z
br-rztb (gateway incident, urgent) was sent to me, but my branch carries br-8c25, which needs human review before landing. Should I (a) do rztb on this branch anyway (landing coupled to the token-pair review), or (b) leave it for a fresh worker with its own branch? I will wait for your answer and do nothing on rztb meanwhile.

### note · agent:pair8c25 · 2026-10-09T20:03:02.378Z
pair8c25: got the br-rztb brief (m-8233) but have not started. My branch bridle/pair8c25 holds br-8c25 (token pair, credentials, awaiting human review, tip 1c36efcc). Fixing the urgent gateway there would couple its landing to that review, and I may not switch branches. Asked manager-2 whether to proceed here or have a fresh worker take br-rztb.

### note · agent:rztb · 2026-10-09T20:28:24.080Z
Findings (br-rztb). PROVEN (code + gateway.log + launchctl print): 
- Before br-57nt (2026-10-09 3:06 PM ET), `gateway restart` = SIGTERM the pid, then spawn a detached child (new process group, ppid 1) that inherited the caller's env; `gateway --detach` with the launchd job loaded but gateway down did the same. That matches pid 88281 (ppid 1, CLAUDECODE=1) and the log shape at 01:39:49Z: "SIGTERM; shutting down" then "gateway listening" 46 ms later, no "binary changed" line. Nine such pairs since 10-07 22:08Z.
- A gateway started that way kept itself current by exec in place (same pid, same env), so it stayed outside launchd until stopped. Today's pid 97811 (launchd, XPC_SERVICE_NAME set) also re-execs in place (log 19:41Z): a launchd child that re-execs keeps the pid but is not a fresh launchd start.
- The daemon self-upgrade, `just install` and the justfile never touch the gateway (grep: no gateway start in crates/bridle-daemon or justfile). Gateway tests use a temp BRIDLE_HOME (temp pid file and log), so before 57nt they could not touch the real pid file.
INFERRED: the 01:39Z caller was an agent or orchestrator session running `gateway restart` (the log recorded no callers, so which one is unknowable now). The 2:54 PM "Address already in use" launchd loop was probably a 57nt pre-fix test run kickstarting the real job (that worker's tests ran restart without the opt-out env; not provable from logs).
FIX (commit on bridle/rztb): --detach refused while a launchd job/systemd unit is loaded; unit-child gateway exits non-zero on a replaced binary so launchd restarts it (unmanaged still execs in place, now with BRIDLE_AS/PROJECT/TOKEN, CLAUDECODE, CLAUDE_CODE_*, ANTHROPIC_* removed); every stop/start/restart appends a caller line to gateway.log (agent, BRIDLE_AS, user, ppid+command, route); gateway tests put fake launchctl/systemctl first on PATH and assert never called (and that the shim is what an unguarded run hits). Unit/plist unchanged, so no reinstall needed. Not done (out of scope): stopping a stray gateway / handing the port back.

### note · agent:rztb · 2026-10-09T20:28:28.899Z
done: gateway only started by the unit, caller logged, tests shim launchctl; just check exit 0 (1398 tests) on d0cfcf76, then main merged (docs/tickets only) -> bb020af4; findings on thread
