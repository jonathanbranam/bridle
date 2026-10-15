---
id: br-7bs2
title: macOS firewall still blocks a freshly launched bridle (gateway) after a self-upgrade, despite the stable signing identity (p88z)
kind: bug
created: 2026-10-15
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-p88z]
---

## The ask

The human, 2026-10-15 ~7:10 PM ET (to the aide), from their phone: "Dalek gateway just shows a bar no response at all." Then, on the firewall diagnosis: "Wtf. Yeah that's probably it. I'll check in a bit. How the hell do we stop this?"

Found (aide, 19:05-19:15 ET):
- dalek's gateway (`bridle gateway`, launchd `dev.bridle.gateway`) restarted onto the new binary at 18:41:57 ET after a self-upgrade (new pid 88900). Binary re-signed 18:41:38 with "bridle local signing" (`codesign -dvvv`: Authority=bridle local signing; designated requirement `identifier bridle and certificate leaf = H"37e5..."`; `security find-identity -p codesigning -v`: 1 valid identity).
- From remus over Tailscale: 7401 (bridle daemon), 7405 (bridle-ui), 7406 (track-web) answer (401); 7878 (gateway) times out. Locally on dalek, 7878 answers 200. No SYN_RCVD seen on dalek during a remote attempt.
- The long-running processes (started Oct 7 / Oct 11, re-exec'd in place) are reachable; only the newly launched process is blocked.
- `socketfilterfw`: firewall on, stealth off, block-all off, "Incoming connection to ~/.cargo/bin/bridle is permitted". No alf log lines found.

So p88z's acceptance ("After the first Allow, a rebuild doesn't prompt again (verify on dalek across a self-upgrade)") doesn't hold. Possible causes to check: the firewall entry was recorded against an older ad-hoc signature and needs one Allow under the new identity (then it would hold from now on); or the firewall keys on something other than the designated requirement for a self-signed leaf.

Ask: find the cause and make an upgrade never leave a bridle listener blocked. If signing can't guarantee it, consider not having bridle accept tailnet connections itself (e.g. listen on 127.0.0.1 and front it with `tailscale serve`, so the firewall never sees bridle as an inbound listener). Also: detect it. The gateway/daemon could self-check reachability on its tailnet address after a restart and raise an incident, instead of the human finding out from a phone.

Workaround now: click Allow on dalek's prompt, or remove and re-add bridle in System Settings > Network > Firewall > Options.
