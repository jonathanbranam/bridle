---
id: br-53gy
title: Take over is a one-shot command, not 'serve --take-over'; plus stop and start a project's persistent daemon on any machine
kind: feature
created: 2026-10-11
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: [br-2wnn]
related: [br-kt25, br-hw6c, br-evy6, br-r2nr]
theme: multi-machine
parent: br-kx5u
resolved: 2026-10-17T02:08:19Z
---

## The ask

The human, 2026-10-11 ~6:40 PM ET (to the aide), verbatim, quoting an agent's NUC instructions:

> > bridle --project <p> serve --take-over doesn't exit once it has claimed the project: it keeps running the daemon until stopped. So for each NUC project the step is:
>
> Take over should just be a one time action not part of serve. We need a way to
>
> 1. Stop the persistent daemon (on any machine)
> 2. Take over / transfer
> 3. Start the persistent daemon

So: taking over a project (moving `owner.toml` on `bridle/state` to this machine, hw6c) becomes a one-shot command that does its job and exits, not a flag on `bridle serve`. And there are commands to stop and start a project's persistent daemon (launchd on macOS, systemd on Linux) on any machine, so a move is stop (old machine), take over, start (new machine).

Related: kt25 (move a project with one command) would be built from these three steps; evy6 (one placement record).
