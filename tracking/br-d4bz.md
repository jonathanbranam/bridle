+++
id = "br-d4bz"
title = "daemon restart reports 'did not come back within 95s' when it came back in seconds"
kind = "bug"
state = "built"
created_at = "2026-11-17T13:38:37.828Z"
updated_at = "2026-11-17T21:07:02.980166Z"
created_by = "external:orchestrator@remus"
watchers = [
    "external:orchestrator@remus",
    "agent:d4bz",
]
branch = "bridle/d4bz"
commit = "35d0fc28"
summary = "wait_for_restart needed to see the daemon down before accepting 'back', so a restart faster than one 500 ms poll timed out. /v1/health now carries started_at (daemon process start; pid is unchanged because it execs); restart takes it before the request and treats a changed value on an answering daemon as back. Down-then-up and timeout-with-log-hint paths kept; tests added; cli.md and CHANGELOG updated."
ticket = "d4bz"
+++

docs/tickets/open/daemon-restart-reports-did-not-come-back-within-95s-when-it-d4bz.md

submitted by external:orchestrator@remus

Seen twice on remus with the flume daemon (127.0.0.1:7412, also bound to its tailnet address): 2026-11-17 04:40 UTC and 13:36 UTC. 'BRIDLE_PROJECT=flume bridle daemon restart' printed 'restarting at <sha>: 1 agent to resume' and then 'error: the daemon did not come back within 95s; see .../daemon.log'. The log shows it back within 2 s (13:36:37 restart requested, 13:36:39 state branch at start), and 'bridle status' worked right away. Guess: the CLI polls an address or identity that doesn't match after the restart (the daemon listens beyond loopback). Impact: a false failure leads an operator to dig into a healthy daemon or restart it again.

## Thread

### note · external:orchestrator@remus · 2026-11-17T13:38:37.829Z
submitted by external:orchestrator@remus

### note · external:orchestrator · 2026-11-17T15:05:39.220Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-17T15:05:39.521Z
Readied by orchestrator (bug, self-evident ask; rule ready-without-asking).

### note · agent:d4bz · 2026-11-17T21:07:02.930Z
built: 35d0fc28 (branch bridle/d4bz)

### note · agent:d4bz · 2026-11-17T21:07:02.980Z
done: restart detects fast restarts via health started_at; 35d0fc28; just check exit 0, 2019 tests passed
