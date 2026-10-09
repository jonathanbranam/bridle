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
self-re-exec stayed under launchd. The open question is the earlier `restart`/`--detach` path.

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
