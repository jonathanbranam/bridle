---
id: br-2fjw
title: Gateway self-connect warning fires after every daemon start though the gateway is reachable
kind: bug
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-11-21T18:40:46Z
---

## The ask

The gateway's "cannot connect to its own address" incident fires after daemon starts although the gateway is reachable. Each false note wakes the orchestrator. Make it fire only when the address really is unreachable.

Observed by the dalek orchestrator (submitted by external:orchestrator), 2026-11-21: after both daemon starts (17:41 and 17:42 UTC) it got the system note "The gateway cannot connect to its own address 100.100.189.100:7878 after a start ... application firewall ..." (m-0807, m-0814). Each time `curl http://100.100.189.100:7878/` from dalek returned 200 within a minute; no firewall dialog was involved. Submitter's guess: the check runs before the listener is up, or tries once with no retry; asked for a retry window of about 30 s.

## What is known (pm-1, from the code)

`crates/bridle-gateway/src/reachable.rs` already retries: `PROBE_WAITS` is 3 s, 10 s, 10 s (about 23 s) via `bridle_api::probe::reachable`, and a peer machine may be asked (`ask`). The text in the note is the `Verdict::Blocked` one (`unreachable_text`), reached when the peers say not reachable or no peer answers and the self-connect fails. So a plain "no retry" is not the cause. Find the real cause first: the gateway listener coming up later than ~23 s after a daemon restart (the daemon restart also restarts or reconnects the gateway?); a peer reporting "blocked" during the restart window; or the probe starting before the listener binds. Check `crates/bridle-api/src/probe.rs`, where the check is called from in `crates/bridle-gateway/src/` (and the daemon's own `probe_own_address` in `crates/bridle-daemon/src/lib.rs` ~line 1446, a sibling message with the same wording), the gateway log on dalek if available, and the two incident events' timestamps against the gateway's listen log line.

## Fix

Whatever the cause shows: start probing only after the listener is bound, and/or lengthen the window (30 s or more total) and require a final failing probe before raising; keep the real-block path (a blocked firewall must still raise) and its tests. Add a test for the cause found. Keep it small (kiss).

## Acceptance

`just check` passes; a test reproducing the false positive (listener binds late, or a transient peer "false" during restart) now raises nothing, and the closed-port case still raises. No migration (behaviour only). Update the design doc line about this check if one exists (grep docs/design for "own address").

## Model

Sonnet. Out of scope: the firewall-prompt research (br-ptu8), the signing chain.
