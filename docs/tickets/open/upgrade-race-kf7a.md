---
id: br-kf7a
title: "Concurrent self-upgrade on one machine: one daemon fails 'installing over ~/.cargo/bin/bridle: No such file'"
kind: bug
created: 2026-11-27
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Self-upgrade must be safe when several daemons on one machine install over the same binary at once, and an upgrade failure message must name the project.

Submitted by orchestrator@vesta (vesta-home). On vesta, 2026-11-27 15:08:59-15:09:10 UTC, every project daemon self-upgraded to release v0.8.1 at once. One reported:

    [upgrade_failed] upgrade: release v0.8.1 not installed; the daemon is unchanged: installing over /home/jbranam/.cargo/bin/bridle: No such file or directory (os error 2)

The others installed it and all daemons restarted at v0.8.1, so it was harmless this time; on other timing a daemon might keep running the old version. The message did not say which project failed.

## Likely cause (pm-1, from the code)

`install_signed` in `crates/bridle-daemon/src/release.rs` (~lines 221-251) stages to one fixed name, `<exe>.new`, then renames it over `exe`. A second daemon renames that file away between the first one's copy and rename, giving ENOENT. Confirm it. Check the other install paths for the same shared name: `signing::install` (`crates/bridle-daemon/src/signing.rs`, the macOS path), the commit-build upgrade path, and `rollback::stash_previous` (what is kept for rollback if daemon B stashes after daemon A already installed the new binary: B would then "roll back" to the new one).

## Suggested fix (smallest that is right)

A unique staged name per process, plus, if the rollback stash needs it, a machine-wide lock around stash and install; if the installed binary already is the target version, treat that as success and go on to restart. Put the project name in the `upgrade_failed` message.

## Acceptance

`just check`; a test with concurrent installs to one path that all succeed and leave a whole binary, and one showing the rollback stash still holds the old binary. Update the upgrade design doc if it describes the install. No migration.

## Model

Sonnet. Out of scope: macOS firewall prompts (br-ptu8), staggering when daemons check for releases.
