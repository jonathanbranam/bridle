+++
id = "br-3p3h"
title = "Every daemon, test daemons included, forks 'ps -axo' every 2 s even with no agents: dozens at once during a test run drive load to 80-95 and hold all spawns"
kind = "bug"
state = "open"
created_at = "2026-10-08T13:04:51.766Z"
updated_at = "2026-10-08T13:04:51.827158Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
priority = "urgent"
priority_at = "2026-10-08T13:04:51.769428Z"
+++

Seen overnight and this morning 2026-10-08: load 76-95 on 16 cores several times, each holding every new spawn (the governor's threshold is 2.5/core); top consumers named (ps) up to 186% and syspolicyd up to 503%.

Cause (checked at 40322d67): lib.rs runs tick_tracker every tracker_interval (2 s, lib.rs:154); supervisor.rs tick_tracker calls containment::snapshot(), which forks `ps -axo pid=,ppid=,pgid=,lstart=`, even when the daemon has no live agent. Every daemon does it, including each test daemon nextest starts: at 9:04 AM there were ~17 test binaries from one worktree (renew_test x5, restart_in_place_test x3, ...) plus the 3 real daemons, each with its own ps in flight, each ps taking 1-2 s on macOS (and an exec syspolicyd looks at). More load makes ps slower, so the runs overlap more.

Fix, in order of payoff:
1. tick_tracker returns early when runtimes is empty (no snapshot with no agents). That alone removes it from idle daemons and almost every test daemon.
2. Test harness: a longer tracker_interval in the test overrides unless a test needs the tracker (containment tests set it themselves).
3. Optional, if 1-2 are not enough: read the process table without forking ps (the sysinfo crate, or /proc on Linux and libproc via a safe wrapper on macOS; unsafe stays forbidden in our code). Keep ps as the fallback.
Acceptance: just check passes; a test that a daemon with no agents never calls snapshot (inject the snapshot fn or count calls); measure load during `just test` before/after on dalek and put the numbers in the done note. Update docs/design/agent-host/agents.md (Containment) to say the tracker runs only while agents are live.

## Thread

### note · external:orchestrator · 2026-10-08T13:04:51.769Z
priority: normal -> urgent
