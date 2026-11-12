+++
id = "br-maj3"
title = "The firewall warning can't fire yet: with no upgraded peer the probe falls back to the same-host connect and passes silently (9qws)"
kind = "bug"
state = "integrated"
created_at = "2026-11-11T21:13:05.371Z"
updated_at = "2026-11-12T03:32:31.540636Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:maj3",
]
branch = "bridle/maj3"
commit = "f99c3012e1c8faebf0e5e46dcc58b69cb19479fa"
summary = """
The self-probe (crates/bridle-api/src/probe.rs) now returns a Verdict (Reachable, Blocked, Unverified) instead of a bool. ask_peers logs at info each peer asked and why one was skipped; reachable logs each attempt and the final verdict. With no peer verdict, a passing same-host connect is Unverified (not a pass); a failing one or a peer's "cannot connect" is Blocked (existing incident). The daemon (probe_own_address) records a lower-grade "reachability unverified" incident without asking the orchestrator; the gateway (reachable::check) raises its unverified text through the same raise path. Both probes run once per start, so at most once. No wire change, no migration. Docs: api.md connect-back row. Tests: reachable, unverified, blocked-peer and closed-port cases in probe, daemon and gateway. Note: the full check had one failure, restart_land_test a_land_past_the_cap... (43s under load 15-29); it passes alone in 5s and is unrelated.
Item 3 (for manager-2 or the aide, after vesta and remus self-upgrade, br-p9sq): restart dalek's gateway, check gateway.log for "probe: peer answered" lines with reachable=true; then deliberately block a listener (a bridle copy under another identifier, the br-7bs2 method) and confirm the blocked incident is raised. If the log says "no peer gave a verdict" the peers still lack connect-back."""
ticket = "maj3"
+++

docs/tickets/open/the-firewall-warning-can-t-fire-yet-with-no-upgraded-peer-th-maj3.md

## Thread

### note · external:aide · 2026-11-11T21:13:05.684Z
readied by external:aide

### note · external:aide · 2026-11-11T21:13:05.790Z
aide: readied at filing (kind bug, rule ready-without-asking).

### note · agent:pm-1 · 2026-11-11T21:13:46.683Z
pm-1 brief: model Sonnet. Build items 1 and 2 of the ask now; item 3 is a check that needs vesta and remus to self-upgrade (br-p9sq, the human's to-do), so it is NOT part of this worker's acceptance: the worker leaves a note on this thread saying what to run once the peers upgrade, and manager-2 or the aide does it then.

Files: crates/bridle-api/src/probe.rs (ask_peers, reachable, self_connect), the callers in crates/bridle-daemon/src/lib.rs (probe_own_address) and crates/bridle-gateway/src/reachable.rs. Read br-9qws's thread and commit 828b189c first.
1. Log at info on every run: each peer asked and its answer or why it was skipped (error text), whether the self-connect fallback was used, and the final verdict. Use tracing.
2. When no peer gives a verdict, do not return "reachable": return an explicit "unverified" result. The daemon and gateway raise a lower-grade incident or warning ("reachability unverified: no peer answered; a same-host connect proves nothing on macOS"), at most once per process start, and never a false "blocked" incident. Keep the existing blocked-incident path for a peer verdict of unreachable.
Tests: no peers answer -> unverified, warned once; a peer answers reachable -> no warning; a peer answers unreachable -> the blocked incident; skipped peers are logged. No wire change (a peer without connect_back is just skipped), so no migration; say so in the CHANGELOG. Docs: where the probe is described in docs/design/agent-host/. Acceptance: just check (run with BRIDLE_* env unset). Out of scope: item 3, tailscale serve, signing.

### note · agent:maj3 · 2026-11-12T00:17:13.357Z
The self-probe (crates/bridle-api/src/probe.rs) now returns a Verdict (Reachable, Blocked, Unverified) instead of a bool. ask_peers logs at info each peer asked and why one was skipped; reachable logs each attempt and the final verdict. With no peer verdict, a passing same-host connect is Unverified (not a pass); a failing one or a peer's "cannot connect" is Blocked (existing incident). The daemon (probe_own_address) records a lower-grade "reachability unverified" incident without asking the orchestrator; the gateway (reachable::check) raises its unverified text through the same raise path. Both probes run once per start, so at most once. No wire change, no migration. Docs: api.md connect-back row. Tests: reachable, unverified, blocked-peer and closed-port cases in probe, daemon and gateway. Note: the full check had one failure, restart_land_test a_land_past_the_cap... (43s under load 15-29); it passes alone in 5s and is unrelated.
Item 3 (for manager-2 or the aide, after vesta and remus self-upgrade, br-p9sq): restart dalek's gateway, check gateway.log for "probe: peer answered" lines with reachable=true; then deliberately block a listener (a bridle copy under another identifier, the br-7bs2 method) and confirm the blocked incident is raised. If the log says "no peer gave a verdict" the peers still lack connect-back.

### note · agent:maj3 · 2026-11-12T00:17:15.320Z
built: 0c4f630ed0a2604ea6bbde965135b7b67b566494 (branch bridle/maj3)

### note · agent:maj3 · 2026-11-12T00:17:15.384Z
done: probe returns Reachable/Blocked/Unverified, logs each run, unverified incident once per start; just check run with env -i: 1684/1685 passed, only restart_land_test a_land_past_the_cap failed (43s under load, passes alone in 5s, unrelated); item 3 note on thread; 0c4f630ed0a2604ea6bbde965135b7b67b566494

### note · agent:maj3 · 2026-11-12T00:51:23.859Z
done: merged main (79b892b6); just check with env -i exit 0, 2003 tests run, 2003 passed, 6 skipped, on efadaef2871bebc989bb3c8e0ebedeb64d09abd0

### note · agent:manager-2 · 2026-11-12T00:51:31.590Z
integrated: f99c3012e1c8faebf0e5e46dcc58b69cb19479fa (branch bridle/maj3)

### note · agent:manager-2 · 2026-11-12T00:51:46.868Z
cleanup: removed agent maj3, origin branch bridle/maj3, branch bridle/maj3

### note · agent:pm-1 · 2026-11-12T03:32:31.540Z
pm-1: reminder for item 3. It is not done and cannot be until vesta and remus self-upgrade (br-p9sq, the human's to-do). Then manager-2 or aide: follow the worker's note above (restart dalek's gateway, look for "probe: peer answered" reachable=true in gateway.log, block a listener the br-7bs2 way, confirm the blocked incident). Until then dalek reports "reachability unverified" and the human's firewall dialogs have no blocked warning.
