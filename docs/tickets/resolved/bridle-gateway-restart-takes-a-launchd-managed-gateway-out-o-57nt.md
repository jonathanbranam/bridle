---
id: 57nt
title: bridle gateway restart takes a launchd-managed gateway out of launchd and inherits the caller's Claude env
kind: bug
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [76td, bek3, ppa6]
tasks: [br-57nt]
closed: 2026-10-09T21:36:10Z
---

## The ask

The human, verbatim (2026-10-08 ~9:50 PM ET, relayed by the bridle-ui aide, m-7456; speech-to-text, "launch D" = launchd):

> the gateway, it's running on launch D. At least I ran on launch D. We went through this already, and you said it had been run from Claude code. I really did never think that was true, and I don't think that's true. I don't think it was ever run from code, frankly. And we restarted it on the launch, and it went back to the way it was before. Is this the upgrade that's causing this, or something else? When we're landing a change, it is relaunching the gateway, at least, as if it's from Claude.

They saw: "Daemon unreachable: no human token for project 'bridle': running inside Claude Code ($CLAUDECODE is set) ...".

The human, verbatim (2026-10-09 ~5:30 PM ET, to the bridle-ui aide):

> there is a ticket to investigate why the gateway changes owners. It may be part of the ugprade process or testing - that behavior is not mine and AFAIK not an active, external agent. I think it is the sytem or part of upgrades. I did kill the spawned gateway and launchd spawned a proper one. There is only one orchestrator per machine; we need to make that clear. the orch for this machine runs in the bridle project.

So: find which part of the system (the upgrade or landing path, or tests) restarts the gateway
outside launchd. It is not the human and, as far as they know, not an external agent acting on
its own. Anything here that names "the orchestrator" means the one orchestrator on dalek, which
runs in the bridle project. No orchestrator belongs to the bridle-ui project.

State at 2026-10-09 5:34 PM ET: launchd's `dev.bridle.gateway` is running (pid 97811). At
21:31:12Z it restarted itself onto a new binary and logged `via="service manager"`, so that
self-re-exec stayed under launchd. The fix for the `restart`/`--detach` path, br-57nt, landed
at 3:06 PM ET (4e28ca82): restart goes through launchd/systemd, and the detached child drops
Claude's environment. Both of the human's guesses were right. The owner changed during
landings and upgrades (`bridle gateway restart` from a Claude session), and a test once
kickstarted the real `dev.bridle.gateway` job (the worker's note; tests now set
`BRIDLE_GATEWAY_UNMANAGED=1`). What's left: confirm that the next landing or upgrade keeps the
gateway under launchd, then resolve.

## Facts (the bridle-ui aide's findings)

- launchd job `dev.bridle.gateway` was "not running", last exit 0, runs=2.
- `gateway.log` 01:39:49Z: "SIGTERM; shutting down", then a new gateway (pid 88281, ppid 1) whose env had `CLAUDECODE=1` and `CLAUDE_CODE_ENTRYPOINT=cli`: started by `bridle gateway restart` / `--detach` (br-76td, br-bek3) from a Claude session, likely during the 9:30-9:40 PM landing and upgrade (br-ppa6 landed 21:30).
- br-ppa6 strips `BRIDLE_AS`/`BRIDLE_PROJECT` but not `CLAUDECODE`; documents failed with the CLAUDECODE error 01:40-01:43Z.
- The 21:54 binary's self-re-exec logs "ignoring CLAUDECODE from the starting shell", so the token error is fixed (unreachable=[] since 01:54Z).
- The human was right that they ran it under launchd; the earlier "run from Claude Code" explanation was the restart replacing it.

## The bug

`bridle gateway restart` kills the launchd-managed gateway and replaces it with a detached child of the caller. launchd stops supervising it (exit 0, so no KeepAlive restart), and the child inherits the caller's environment.

## The ask

- When launchd (or systemd) manages the gateway, restart through it (`launchctl kickstart -k gui/<uid>/dev.bridle.gateway`), or refuse and say so.
- Never inherit Claude's environment.
- One supervision model for the gateway.

## Resolution

br-57nt landed 2026-10-09 3:06 PM ET (4e28ca82). Seen working (advisor product-manager, at the
human's request via the bridle-ui aide): at 5:31 PM ET (21:31Z) the installed binary changed and
`~/.bridle/gateway.log` shows "the bridle binary changed; restarting onto it", then "gateway
starting via=\"service manager\""; `launchctl list` shows `dev.bridle.gateway` holding pid 97811
(ppid 1). The recurring restart outside launchd is incident rztb.
