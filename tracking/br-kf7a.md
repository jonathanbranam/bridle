+++
id = "br-kf7a"
title = "Concurrent self-upgrade on one machine: one daemon fails 'installing over ~/.cargo/bin/bridle: No such file'"
kind = "bug"
state = "pending"
created_at = "2026-11-27T15:09:51.749Z"
updated_at = "2026-11-27T15:09:51.750563Z"
created_by = "external:orchestrator@vesta"
watchers = ["external:orchestrator@vesta"]
+++

submitted by external:orchestrator@vesta

## What happened

On vesta, 2026-11-27 15:08:59-15:09:10 UTC, every project daemon self-upgraded to release v0.8.1 at the same time. The forwarded system messages to external:orchestrator@vesta-home included one:

    [upgrade_failed] upgrade: release v0.8.1 not installed; the daemon is unchanged: installing over /home/jbranam/.cargo/bin/bridle: No such file or directory (os error 2)

The other daemons installed it, and ~/.cargo/bin/bridle is v0.8.1 (mtime 15:09 UTC). All daemons restarted at v0.8.1 and resumed their agents. The forwarded message did not say which project's daemon failed.

## Likely cause

Several daemons on one machine install over the same shared binary path at once. One daemon's replace (a rename or unlink) races another, so the path briefly doesn't exist. The race was harmless this time, but the failing daemon reports a failure that didn't matter, and on a different timing a daemon might keep running the old version.

## The ask

Make self-upgrade safe when several daemons share one binary: either one install per machine (a lock, or skip when the installed binary already matches the target version), or treat "already at target version" as success. Include the project name in the upgrade_failed message.

Filed by orchestrator (vesta-home).

## Thread

### note · external:orchestrator@vesta · 2026-11-27T15:09:51.750Z
submitted by external:orchestrator@vesta
