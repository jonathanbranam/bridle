+++
id = "br-9qws"
title = "Verify the firewall self-probe can fire: a self-connect from the same host may never be filtered"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T21:14:26.820Z"
updated_at = "2026-11-11T14:41:00.836613Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:9qws",
]
branch = "bridle/9qws"
commit = "1660fdd4c5076347f03aed6d657a3e4a1a0aa367"
summary = "The 2026-11-09 logs show both the gateway and daemon self-probes ran and passed while the binary was blocked (no error line, and every attempt finished before the Allow clicks), so a same-host self-connect is not filtered. Added `POST /v1/connect-back {addr}` (peer token only; types in bridle-api, `Client::connect_back`, handler in server.rs, api.md row) and `bridle_api::probe` (`ask_peers` asks daemons on other machines in turn; `reachable` uses a peer verdict, else falls back to the old self-connect). Daemon `probe_own_address` and gateway `reachable::check` now use it; incident unchanged. Caveat: with no peer machine configured the probe is still the weak self-connect, so a single-machine setup is not covered. Tests: blocked-peer raises though self-connect passes (daemon, gateway, api), peer-ok, no-peer fallback. Not tested end to end against a real second machine."
ticket = "9qws"
+++

docs/tickets/open/verify-the-firewall-self-probe-can-fire-a-self-connect-from-9qws.md

## Thread

### note · external:aide · 2026-10-16T21:42:46.471Z
The human, 2026-10-16 ~5:35 PM ET (via aide), approving tvrp, whce, 8kzq, 9qws, vmrc, r8jf: "Yes that looks good." Readied by aide.

### note · agent:manager-2 · 2026-10-21T10:56:42.505Z
held: Needs the human's Mac: a deliberately blocked binary to see whether the self-connect probe fires (ask via the manager; never disable the firewall). No worker can do the first step alone. (until the human offers the Mac for the blocked-binary test)

### note · external:aide · 2026-11-11T13:09:34.355Z
aide: the human, 2026-11-11 (to aide), verbatim: "Yeah, that's good. Let's go with C to do the warning." They chose to live with the firewall prompts (br-qy4y) and to have a warning when bridle is blocked. This ticket is that warning, so it should be unheld. The hold waited for the human's Mac for a staged blocked-binary test. 11-09 may already be that test: the gateway started blocked at 17:11:11Z (gateway.log), the human's Allow clicks are logged at 17:12:06-08Z (br-qy4y findings), and no incident fired. Check the gateway's and the daemons' self-probe timing in that window. If a probe ran before 17:12:06Z and passed, a self-connect isn't filtered, and the probe has to come from a peer as the ask says. Only if that can't be shown does the staged test (and the human's Mac) come back.

### note · agent:pm-1 · 2026-11-11T13:09:44.469Z
pm-1 revised brief (the human chose option C on br-qy4y, 2026-11-11 via aide: live with the prompts, make the warning fire). Supersedes the "needs the human's Mac" step; unhold.

Step 1, from logs only (no firewall change, no human Mac): the 2026-11-09 upgrade is a natural blocked-binary test. The gateway started blocked at 17:11:11Z (gateway.log), the human's Allow clicks are logged 17:12:06-08Z (br-qy4y findings). Read the gateway's and each daemon's self-probe log lines (probe_own_address in crates/bridle-daemon/src/lib.rs, crates/bridle-gateway/src/reachable.rs) in that window. Record the finding on this thread: did a probe run before 17:12:06Z, and did it pass? If it passed while the binary was blocked, a same-host self-connect is not filtered, so change the probe to come from a peer (ask a peer daemon over the tailnet to connect back, or reuse an existing peer call) and keep the incident; with tests. If no probe ran in the window or the logs can't show it, say so and what the cheapest honest test is (a second `bridle` listener under a different identifier on a spare port, which raises a dialog: ask on this thread first, never disable the firewall).
Acceptance: the finding recorded; if the probe changed, `just check` and tests for a blocked-peer case. Model: Sonnet. br-t9j4 is landed (the gateway's own check exists). Out of scope: tailscale serve, signing, br-p29s.

### note · agent:manager-2 · 2026-11-11T13:09:52.375Z
unheld

### note · agent:manager-2 · 2026-11-11T13:10:11.769Z
manager-2: unheld and given to worker 9qws (Sonnet). br-wt4f (Mac test for the human) is already marked done by the human (2026-10-18) and is superseded by step 1 (logs) unless the logs cannot settle it; nothing to close. br-qy4y is resolved.

### note · agent:9qws · 2026-11-11T13:10:33.756Z
Step 1 finding (logs only, 2026-11-09 window). Both probes ran and passed while the binary was blocked, so a same-host self-connect is not filtered.
- gateway.log: SIGTERM 17:11:00.99Z, gateway listening on 100.100.189.100:7878 at 17:11:11.606Z. The probe (reachable::check) waits 3s/10s/10s and each connect has a 5s timeout, so every attempt finished by ~17:11:45Z, before the Allow clicks at 17:12:06-08Z. No "cannot reach its own address" line anywhere in gateway.log, so one attempt connected. A success logs nothing, which is why this is inferred from the absence of the error.
- bridle daemon.log: listens on 100.100.189.100:7401 at 17:11:04.6Z; probe_own_address ran 17:11:07Z to ~17:11:40Z; no "daemon cannot reach its own address" either.
- Caveat: the logs show the gateway "started blocked" only via the human's dialogs (br-qy4y); nothing logs the block itself. But the dialog was pending for the gateway for the whole probe window, and the probe passed. The simplest explanation is that connecting to a local address goes over loopback and the application firewall does not filter it.
Decision: change both probes to come from a peer (a peer daemon connects back over the tailnet), keep the incident.

### note · agent:9qws · 2026-11-11T14:40:49.030Z
built: 828b189c (branch bridle/9qws)

### note · agent:9qws · 2026-11-11T14:40:49.171Z
done: self-probe now asks a peer machine to connect back (logs showed same-host probes passed while blocked); just check exit 0, 1995 tests; 828b189c

### note · agent:manager-2 · 2026-11-11T14:41:00.836Z
integrated: 1660fdd4c5076347f03aed6d657a3e4a1a0aa367 (branch bridle/9qws)
