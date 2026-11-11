---
id: br-maj3
title: "The firewall warning can't fire yet: with no upgraded peer the probe falls back to the same-host connect and passes silently (9qws)"
kind: bug
created: 2026-11-11
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

Filed by aide. Follow-up to br-9qws (landed 1660fdd4). Related: br-qy4y (the human chose to live with the firewall prompts if a warning fires), br-p9sq (vesta and remus self-upgrade).

The human, 2026-11-11 ~4:15 PM ET (to aide), verbatim: "i just had 3 bridle firewall prompts waiting for me"

The human's decision on br-qy4y, 2026-11-11 (to aide), verbatim: "Yeah, that's good. Let's go with C to do the warning."

## What happened

At 2026-11-11 20:35:09Z, dalek self-upgraded (the br-k3ux landing) onto a binary that includes br-9qws, and restarted the gateway, bridle-ui and track-web via launchd. The human then found 3 firewall dialogs pending. No incident was raised on bridle, bridle-ui, track-web or dalek-home, and none of the logs (gateway.log, the daemon logs) say anything about the probe.

Likely why, from crates/bridle-api/src/probe.rs: `ask_peers` asks each peer daemon for `connect_back` and quietly skips every peer that errors. The peers are vesta and remus, which haven't self-upgraded (br-p9sq), so they don't have the new connect-back endpoint. With no peer verdict, `reachable` falls back to `self_connect`, which 9qws itself showed passes while the firewall is blocking. So the check passed silently, and the warning the human chose can't fire until another machine runs the new version. Nothing is logged on a pass, or on the path taken.

## The ask

The warning fires when a bridle listener is blocked, and says when it can't tell:
1. Log the probe's path and result at info on every run (which peer answered or why each was skipped; self-connect fallback; verdict).
2. When no peer gives a verdict, don't report "reachable": a same-host connect proves nothing on macOS. Raise a lower-grade incident or warning ("reachability unverified: no peer answered") instead of silence, at most once per start.
3. Check after the peers upgrade (br-p9sq) that a blocked listener on dalek really raises the incident.
