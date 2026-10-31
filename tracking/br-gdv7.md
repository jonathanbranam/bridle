+++
id = "br-gdv7"
title = "bridle migrate (and daemon list) can't find the running daemon from agent sessions: 'no running daemon registered'"
kind = "bug"
state = "open"
created_at = "2026-10-31T10:17:00.721Z"
updated_at = "2026-10-31T10:17:20.524906Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
