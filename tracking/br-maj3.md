+++
id = "br-maj3"
title = "The firewall warning can't fire yet: with no upgraded peer the probe falls back to the same-host connect and passes silently (9qws)"
kind = "bug"
state = "open"
created_at = "2026-11-11T21:13:05.371Z"
updated_at = "2026-11-11T21:13:46.683863Z"
created_by = "external:aide"
watchers = ["external:aide"]
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
