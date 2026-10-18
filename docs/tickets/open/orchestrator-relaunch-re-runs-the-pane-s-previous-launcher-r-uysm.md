---
id: br-uysm
title: Orchestrator relaunch re-runs the pane's previous launcher (respawn-pane with no command), so a moved orchestrator came back as --project bridle
kind: bug
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-8uxe, br-vynr]
---

## The ask

The human, 2026-10-17 ~10:05 PM ET (via aide), asked whether they had relaunched the moved
orchestrator by hand: "It came back itself".

## What happened

Moving dalek's orchestrator from bridle to dalek-home (br-8uxe): `[orchestrator]` moved to
dalek-home's config, both daemons restarted, then the human `/exit`ed the orchestrator in pane %4.
dalek-home's supervisor relaunched it (its daemon.log, 01:58:40Z: `relaunching the orchestrator
pane=%4 attempt=1 free=false`), but the pane now runs
`zsh -c bridle session orchestrator --project bridle; exec "${SHELL:-sh}"`, with cwd in the
bridle repo. It works (its waiter is on dalek-home), but the session is bridle's, not dalek-home's.

## Cause (read from the code, not reproduced)

`Tmux::respawn(pane, None)` in `crates/bridle-daemon/src/orchestrator.rs` runs
`tmux respawn-pane -k -t <pane>` and its doc says that gives "the pane's shell". It doesn't: with
no command, tmux re-runs the command the pane was created with or last respawned with. Pane %4
had earlier been respawned by bridle's supervisor with the direct form (br-vynr: attempt 2+,
`<launcher>; exec "${SHELL:-sh}"` with `--project bridle`), so every later relaunch, from any
daemon, re-runs bridle's launcher, and the typed `--project dalek-home` line goes nowhere useful.
The next handover relaunch will do the same again, so the orchestrator stays bridle's until this
is fixed.

## Ask

A first-attempt respawn starts a plain shell explicitly (e.g. pass `"${SHELL:-sh}"` / the user's
login shell as the command), so the typed launcher is the one that runs. Test: a pane whose last
respawn command was another launcher gets this daemon's launcher after a relaunch.
