+++
id = "br-9qws"
title = "Verify the firewall self-probe can fire: a self-connect from the same host may never be filtered"
kind = "fix"
state = "claimed"
created_at = "2026-10-16T21:14:26.820Z"
updated_at = "2026-11-11T13:10:11.769928Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:9qws",
]
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
