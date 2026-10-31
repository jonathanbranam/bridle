+++
id = "br-gdv7"
title = "bridle migrate (and daemon list) can't find the running daemon from agent sessions: 'no running daemon registered'"
kind = "bug"
state = "integrated"
created_at = "2026-10-31T10:17:00.721Z"
updated_at = "2026-10-31T10:53:17.922565Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wgdv7",
]
branch = "bridle/wgdv7"
commit = "31a1bcf43f11d8ed37c93b54365dd3093d6a1c01"
summary = "daemons falls back to the daemon status finds (BRIDLE_URL/token) when the registry is empty; migrate with an unregistered --project/BRIDLE_PROJECT falls through to the cwd repo via client_for. Other list_registry users untouched. Test in cli_e2e.rs. Doc: daemon.md Discovery. Caveat: migrate's dry run still hits the protected-main guard on main (unchanged)."
ticket = "gdv7"
+++

docs/tickets/open/bridle-migrate-and-daemon-list-can-t-find-the-running-daemon-gdv7.md

submitted by external:orchestrator

Seen on dalek-home, 2026-10-31 ~10:05-10:16 UTC, daemon 0.7.6 restarted at 88d6f6e (manager message m-0548 on dalek-home).

## What happened

Opt-in migrations ticket da-t26q: the manager and its worker both ran `bridle migrate --only 0002-standard-ticket-fields --dry-run` and got:

    no running daemon registered for project dalek-home

from the worker's worktree and the manager's clone. `bridle daemon list` in those sessions says `no daemons running`, yet `bridle status` reaches the daemon fine.

The same dry run from the orchestrator's session (outside the agent sandbox, same clone) works, and I ran the three migrations from there (dalek-home 17aceb1).

So the daemon registry (`~/.bridle/daemons`) is likely not visible to agents (sandbox or HOME), while the status path discovers the daemon another way.

## Why it matters

Every project's opt-in migrations ticket (bridle files one per project on upgrade, e.g. track-web tw-yxux) will block the same way when an agent works it.

## The ask

Make `bridle migrate` (and `daemon list`) find the daemon from inside an agent session the same way `bridle status` does, or let agents read the registry.

## Verify

From a spawned worker, `bridle migrate --dry-run` lists what it would apply instead of failing.

## Thread

### note · external:orchestrator · 2026-10-31T10:17:00.721Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-31T10:17:20.524Z
Readied by orchestrator (bug; rule ready-without-asking).

### note · agent:wgdv7 · 2026-10-31T10:52:55.904Z
built: 5285dac58d2e62aede8fc3dae55d6190a24458ea (branch bridle/wgdv7)

### note · agent:wgdv7 · 2026-10-31T10:52:55.941Z
done: daemons and migrate find the daemon via BRIDLE_URL without a registry; just check exit 0 (BRIDLE_* unset), 1983 tests passed, 5285dac58d2e62aede8fc3dae55d6190a24458ea

### note · agent:manager-2 · 2026-10-31T10:53:03.722Z
integrated: 31a1bcf43f11d8ed37c93b54365dd3093d6a1c01 (branch bridle/wgdv7)

### note · agent:manager-2 · 2026-10-31T10:53:17.922Z
cleanup: removed agent wgdv7, origin branch bridle/wgdv7, branch bridle/wgdv7
