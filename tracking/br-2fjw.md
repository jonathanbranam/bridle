+++
id = "br-2fjw"
title = "Gateway self-connect warning fires after every daemon start though the gateway is reachable"
kind = "bug"
state = "built"
created_at = "2026-11-21T17:43:19.574Z"
updated_at = "2026-11-21T18:22:17.678245Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w2fjw",
]
branch = "bridle/w2fjw"
commit = "7c17b47e"
summary = "Cause not provable from code or logs (none available): only the LAST probe attempt decides Blocked, and the window was ~23 s, so a slow tailnet path or a peer false near the end of a restart raised a false incident. Fix: one shared PROBE_WAITS in bridle_api::probe (3,10,10,15,15 = ~53 s) used by gateway and daemon (two copies removed). Real-block tests kept; added tests for transient peer false that clears and a late-binding listener. No design doc line mentions the window (grep 'own address' found none)."
ticket = "2fjw"
+++

docs/tickets/open/gateway-self-probe-2fjw.md

submitted by external:orchestrator

Observed on dalek 2026-11-21: after both daemon starts (17:41 and 17:42 UTC) the orchestrator got the system note 'The gateway cannot connect to its own address 100.100.189.100:7878 after a start ... application firewall ...' (m-0807, m-0814). Each time, curl http://100.100.189.100:7878/ from dalek returned 200 within a minute; no firewall dialog was involved. The check likely runs before the gateway listener is up, or tries once with no retry. Ask: retry the self-connect for a short window (e.g. 30 s) before sending the note, so it fires only when the gateway is really unreachable. Each false note wakes the orchestrator.

## Thread

### note · external:orchestrator · 2026-11-21T17:43:19.574Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-11-21T17:43:54.177Z
readied by external:orchestrator

### note · agent:w2fjw · 2026-11-21T18:22:12.360Z
built: 7c17b47e (branch bridle/w2fjw)

### note · agent:w2fjw · 2026-11-21T18:22:12.409Z
done: shared probe window ~53 s, tests added; 7c17b47e; just check exit 0, 2025 tests run (2025 passed)

### note · agent:manager-2 · 2026-11-21T18:22:17.678Z
Diff looks fine. main moved (br-9x55 landed). Please merge main into your branch, rerun env -i HOME=$HOME PATH=$PATH just check, commit, and message me the new sha, exit status and test count.
