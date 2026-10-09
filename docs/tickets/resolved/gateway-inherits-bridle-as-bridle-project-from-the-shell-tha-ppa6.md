---
id: ppa6
title: "Gateway inherits BRIDLE_AS / BRIDLE_PROJECT from the shell that starts it: started from the orchestrator's session it calls every daemon as the orchestrator, and the web UI shows every project unreachable"
kind: bug
opened: 2026-10-08
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-ppa6]
closed: 2026-10-09T23:11:08Z
---

## The ask

The gateway (and `bridle gateway restart` / `--detach`) must not act as whatever principal
the starting shell names. Either ignore `BRIDLE_AS` and `BRIDLE_PROJECT` (the gateway calls every
daemon in the registry with the human's tokens; it has no business with one project or one
principal), or refuse to start with them set and say why. Strip them before the `--detach`
re-exec too, so the detached child can't inherit them.

## What happened

An orchestrator session restarted the gateway at 10:37 AM ET on 2026-10-08 (pid 55759). The
orchestrator's launcher sets `BRIDLE_AS=orchestrator` and `BRIDLE_PROJECT=bridle`, and the
gateway inherited both (`ps eww 55759` showed them). From then on it called every daemon as
the orchestrator, and `~/.bridle/gateway.log` showed every project unreachable on every poll:

```
polled interactions new=0 unreachable=["bridle", "bridle-ui", "dalek", "nuc", "track-web"]
```

The human saw this on the web UI, about eight hours later, with no to-dos (aide's report, m-7205):

> "bridle is unreachable: no token for principal 'orchestrator' on project 'bridle' on machine 'dalek' ..."

Aide also noted that any reply sent from the web UI's reply box in that window would have been
written as the orchestrator, if it got through at all.

Fixed by hand at 22:46Z: `env -u BRIDLE_AS -u BRIDLE_PROJECT bridle gateway restart` (pid 43224).
After that, the first poll showed `new=59` and only the four projects that were already
unreachable before (`dotfiles-local`, `meta-notes`, `notes`, `nuc`).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
