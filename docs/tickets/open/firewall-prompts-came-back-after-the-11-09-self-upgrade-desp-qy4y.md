---
id: br-qy4y
title: Firewall prompts came back after the 11-09 self-upgrade despite the stable signing identity (7bs2 again)
kind: bug
created: 2026-11-10
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

Filed by aide. Related: br-7bs2 (resolved), br-p88z, br-t9j4 (open: the gateway checks its own tailnet listener), br-9qws (open: a self-connect may never be filtered).

The human, 2026-11-09 ~12:15 PM ET (to aide), verbatim: "I just got a seequence of permission prompts for bridle to accept incoming connections. I thought this was fixed."

## What happened

On dalek, 2026-11-09 17:11:00Z (12:11 PM ET), the bridle daemon self-upgraded (v0.7.6, after br-4fpk landed). Its log (/Volumes/Data/work/bridle/.bridle/daemon.log):

    installed binary signature before restart exe=~/.cargo/bin/bridle identifier="bridle" requirement=identifier bridle and certificate leaf = H"37e545f62ca15230bdd4d0adec25862c80e0e6c3"
    restarted launchd jobs onto the upgraded binary jobs=["dev.bridle.gateway", "dev.bridle.mail.bridle", "dev.bridle.bridle-ui", "dev.bridle.track-web", "dev.bridle.mail.dalek-home", "dev.bridle.mail.data-contracts"]

So the signing worked as 7bs2 designed it: the stable "bridle local signing" identity (`security find-identity -p codesigning`: 37E545F6...), identifier `bridle`, and every launchd job runs ~/.cargo/bin/bridle. Checked afterwards (aide, 2026-11-09 evening):
- `socketfilterfw --listapps`: one entry, ~/.cargo/bin/bridle, "Allow incoming connections". `--getappblocked`: permitted.
- Listeners (all ~/.cargo/bin/bridle): gateway 7878, bridle-ui 7405 and track-web 7406 (newly launched by the upgrade), plus 7401, 7407 and 7410 (long-running, not relaunched).
- No incident was raised: the daemon's post-restart tailnet self-check didn't fire (see 9qws for why a self-connect may never be filtered), and the gateway has no check of its own (t9j4 b).

That is the same pattern 7bs2 recorded: newly launched processes prompt, long-running re-exec'd ones don't, even with a stable designated requirement. 7bs2's fixes (identifier, sign a copy and rename) didn't stop the prompts.

## The ask

A self-upgrade must not show the human firewall prompts or leave a bridle listener blocked. Find out why a newly launched process with an unchanged designated requirement still prompts (for example, the firewall doesn't trust a self-signed leaf and falls back to the cdhash). If signing can't guarantee it, take 7bs2's fallback: bridle listens only on 127.0.0.1 and `tailscale serve` fronts the tailnet ports, so the firewall never sees bridle as an inbound listener. Detect it too (t9j4 b): an incident, not the human finding out from a dialog.
