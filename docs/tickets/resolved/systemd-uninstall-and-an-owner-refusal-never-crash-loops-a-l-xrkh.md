---
id: xrkh
title: systemd uninstall, and an owner refusal never crash-loops a launchd or systemd unit after a project moves
kind: bug
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [hw6c, 24mj, v7ug]
tasks: [br-xrkh]
closed: 2026-10-09T23:11:03Z
---

## The ask

The human, 2026-10-09 ~10:15 AM ET, verbatim (to advisor product-manager): "Put tickets in there that would enable basically efficient and pretty direct setup for a new machine. ... I want that as a priority so that I can use it to set up the new Windows machine and add it to the network. So it would include everything for configuring background daemons, launching tokens between machines, tokens between projects, and then as a as a additional feature I would like to see if we can get it in is automated project transfer between machines."

Workstream: machine setup. Found by the PdM's survey (not verified on a machine; the worker checks first):
1. There is `bridle launchd uninstall` but no `bridle systemd uninstall`.
2. After a project moves, the old machine's launchd plist (RunAtLoad, KeepAlive) or systemd unit (Restart=on-failure) starts the daemon again; `serve` refuses because the project's owner is now another machine (owner.toml), exits non-zero, and the supervisor likely restarts it in a loop.
3. A systemd unit can't do `serve --take-over`, so a first start on a new owner must be by hand.
The ask: add `bridle systemd uninstall`; make an owner refusal final for the supervisor (a distinct exit code with `RestartPreventExitStatus=` in the unit and the launchd equivalent, or exit 0 with a clear log), so it never loops; document the take-over-then-enable order in cli.md. Acceptance: just check; tests for the exit path and the unit text.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
